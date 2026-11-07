using System;

namespace PezMax.Core.Reading;

/// <summary>
/// 闭区间页范围（0 基）。阅读器的「可见范围」「渲染窗口」「要释放的范围」都用它表示，
/// 空范围用 <see cref="Empty"/>（<c>Last &lt; First</c>），不用 -1 / null 之类的哨兵。
/// </summary>
public readonly struct PageSpan : IEquatable<PageSpan>
{
    /// <summary>空范围。<c>Count == 0</c>，任何 <see cref="Contains"/> 都为 false。</summary>
    public static readonly PageSpan Empty = new(0, -1);

    public PageSpan(int first, int last)
    {
        First = first;
        Last = last;
    }

    public int First { get; }

    public int Last { get; }

    public bool IsEmpty => Last < First;

    public int Count => IsEmpty ? 0 : Last - First + 1;

    public bool Contains(int index) => !IsEmpty && index >= First && index <= Last;

    public static PageSpan Single(int index) => new(index, index);

    /// <summary>按给定的上下界构造；<paramref name="last"/> 小于 <paramref name="first"/> 时得到 <see cref="Empty"/>。</summary>
    public static PageSpan Range(int first, int last) => last < first ? Empty : new PageSpan(first, last);

    public bool Equals(PageSpan other) => First == other.First && Last == other.Last;

    public override bool Equals(object? obj) => obj is PageSpan other && Equals(other);

    public override int GetHashCode() => (First * 397) ^ Last;

    public override string ToString() => IsEmpty ? "[]" : "[" + First + ", " + Last + "]";
}
