using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using PezMax.Core.Api;
using PezMax.Core.Models;
using PezMax.Core.Notify;
using PezMax.Core.State;

namespace PezMax.Shell
{
    /// <summary>
    /// 通知的会话级状态：两条 feed 的 hash、本地已读 / 已弹过、以及 feed 缓存。
    /// 判定全在 Core（`NotificationPolicy` / `FeedCache`，跑 `spec/fixtures/notification/`），
    /// 这里只负责取数、存盘与把结果交回界面。
    /// </summary>
    internal sealed class NotificationCenter
    {
        private readonly Func<string?, Task<string?>> _readLocal;
        private readonly Func<string, Task> _writeLocal;

        private NoticeLocalState _state = new();
        private string? _popupHash;
        private string? _scrollHash;

        public NotificationCenter(Func<string?, Task<string?>> readLocal, Func<string, Task> writeLocal)
        {
            _readLocal = readLocal;
            _writeLocal = writeLocal;
        }

        public IReadOnlyList<NotificationItem> PopupItems { get; private set; } = Array.Empty<NotificationItem>();
        public IReadOnlyList<NotificationItem> ScrollItems { get; private set; } = Array.Empty<NotificationItem>();

        public long UserId { get; private set; }

        public async Task LoadAsync(long userId, CancellationToken cancellationToken = default)
        {
            UserId = userId;
            if (userId <= 0)
            {
                PopupItems = Array.Empty<NotificationItem>();
                ScrollItems = Array.Empty<NotificationItem>();
                return;
            }

            _state = NoticeStorage.Parse(await _readLocal(NoticeStorage.FileName).ConfigureAwait(false));

            PopupItems = await LoadFeedAsync(
                hash => AppServices.Notifications.GetPopupAsync(userId, hash, cancellationToken),
                PopupItems,
                _popupHash,
                (hash, items) => _popupHash = hash).ConfigureAwait(false);

            ScrollItems = await LoadFeedAsync(
                hash => AppServices.Notifications.GetScrollAsync(hash, cancellationToken),
                ScrollItems,
                _scrollHash,
                (hash, items) => _scrollHash = hash).ConfigureAwait(false);
        }

        /// <summary>弹窗队列：type 1/4 弹过的不再弹，其余每次都要提醒（Core 的 popup-plan 规则）。</summary>
        public IReadOnlyList<NotificationItem> PlanPopup()
        {
            var seen = new NoticeKeySet();
            foreach (var key in _state.Popped.Items) seen.Mark(key.UserId, key.NotifyId);
            return NotificationPolicy.PlanPopup(PopupItems, seen, UserId);
        }

        /// <summary>滚动条要展示的通知：先过一遍通道兜底（别人的下架通知不出现）。</summary>
        public IReadOnlyList<NotificationItem> PlanScroll() =>
            ScrollItems.Where(item => NotificationPolicy.ShouldShow(item, UserId, NotifyChannel.Scroll)).ToList();

        public bool IsRead(long notifyId) => _state.IsRead(UserId, notifyId);

        public int UnreadCount => PopupItems.Concat(ScrollItems).Count(item => !IsRead(item.NotifyId));

        public async Task MarkReadAsync(NotificationItem item)
        {
            if (_state.IsRead(UserId, item.NotifyId)) return;
            _state.MarkRead(UserId, item.NotifyId);
            await SaveAsync().ConfigureAwait(false);
        }

        /// <summary>用户关掉一条通知：记成已读，type 1/4 同时记进「已弹过」。</summary>
        public async Task MarkDismissedAsync(NotificationItem item)
        {
            _state.MarkRead(UserId, item.NotifyId);
            if (NotificationPolicy.IsOneShot(item.NotifyType)) _state.MarkPopped(UserId, item.NotifyId);
            await SaveAsync().ConfigureAwait(false);
        }

        public async Task SaveAsync() =>
            await _writeLocal(NoticeStorage.ToJson(_state)).ConfigureAwait(false);

        private static async Task<IReadOnlyList<NotificationItem>> LoadFeedAsync(
            Func<string?, Task<NotificationFeed>> fetch,
            IReadOnlyList<NotificationItem> cached,
            string? previousHash,
            Action<string?, IReadOnlyList<NotificationItem>> remember)
        {
            try
            {
                var feed = await fetch(previousHash).ConfigureAwait(false);
                var reply = new FeedResponse(true, feed.Unchanged, feed.Hash, feed.Items != null);
                var local = new FeedCacheState(previousHash, cached.Count > 0);
                var decision = FeedCache.Decide(local, reply);

                switch (decision.Action)
                {
                    case FeedAction.Store:
                        remember(feed.Hash, feed.Items);
                        return feed.Items;

                    case FeedAction.Keep:
                        return cached;

                    case FeedAction.Refetch:
                    {
                        var fresh = await fetch(null).ConfigureAwait(false);
                        remember(fresh.Hash, fresh.Items);
                        return fresh.Items;
                    }

                    default:
                        return cached;
                }
            }
            catch (Exception)
            {
                // 通知取不到不该挡住界面：滚动条不显示、铃铛保持旧计数。
                return cached;
            }
        }
    }
}
