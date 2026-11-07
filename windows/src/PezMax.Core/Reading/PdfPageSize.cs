using System;

namespace PezMax.Core.Reading;

/// <summary>
/// 一页的尺寸（逻辑单位，只用宽高比）。来源是各端解析器给的页面尺寸：
/// Windows 是 <c>PdfPage.Size</c>，Linux / Android 是渲染器报的页宽页高。
/// </summary>
public readonly struct PdfPageSize
{
    /// <summary>A4 竖版兜底尺寸（595×842 点）。</summary>
    public const double DefaultWidth = 595;

    /// <summary>A4 竖版兜底高度。</summary>
    public const double DefaultHeight = 842;

    public PdfPageSize(double width, double height)
    {
        Width = IsUsable(width) ? width : DefaultWidth;
        Height = IsUsable(height) ? height : DefaultHeight;
    }

    public double Width { get; }

    public double Height { get; }

    /// <summary>高宽比。按显示宽度排连续视图时，页高 = 宽高比 × 显示宽度。</summary>
    public double Aspect => Height / Width;

    /// <summary>归一化后的尺寸；<c>default(PdfPageSize)</c>（宽高皆 0）在这里变成 A4。</summary>
    public PdfPageSize Normalized => new(Width, Height);

    private static bool IsUsable(double value) =>
        !double.IsNaN(value) && !double.IsInfinity(value) && value > 0;
}
