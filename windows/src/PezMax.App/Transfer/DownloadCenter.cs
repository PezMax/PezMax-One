using System;
using System.Threading;

namespace PezMax.Transfer
{
    /// <summary>
    /// 下载的当前状态广播：外壳的浮层靠它显示进度、取消、完成 / 失败的提示。
    /// 队列本身在 <see cref="DownloadService"/>（串行），这里只管「现在在下一个什么、能不能取消」。
    /// </summary>
    internal static class DownloadCenter
    {
        private static CancellationTokenSource? _current;
        private static string _fileName = "";

        public static event Action<string>? Started;
        public static event Action<DownloadProgress>? Progress;
        public static event Action<string>? Completed;
        public static event Action<string, string>? Failed;

        public static bool IsBusy => _current != null;
        public static string CurrentFileName => _fileName;

        /// <summary>取消当前下载；半成品会被保留，下次点「继续下载」还能接上。</summary>
        public static void Cancel()
        {
            try
            {
                _current?.Cancel();
            }
            catch (ObjectDisposedException)
            {
            }
        }

        internal static CancellationTokenSource Begin(string fileName, CancellationToken external)
        {
            var cts = CancellationTokenSource.CreateLinkedTokenSource(external);
            _current = cts;
            _fileName = fileName;
            Started?.Invoke(fileName);
            return cts;
        }

        internal static void Report(DownloadProgress progress) => Progress?.Invoke(progress);

        internal static void Complete(CancellationTokenSource cts, string fileName)
        {
            End(cts);
            Completed?.Invoke(fileName);
        }

        internal static void Fail(CancellationTokenSource cts, string fileName, string message)
        {
            End(cts);
            Failed?.Invoke(fileName, message);
        }

        private static void End(CancellationTokenSource cts)
        {
            if (ReferenceEquals(_current, cts))
            {
                _current = null;
                _fileName = "";
            }

            cts.Dispose();
        }
    }
}
