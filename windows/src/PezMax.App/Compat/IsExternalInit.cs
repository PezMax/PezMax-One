namespace System.Runtime.CompilerServices
{
    // UWP 目标（.NET Core 5.0）没有 init-only 所需的编译器标记；C# 10 的 init 访问器靠它。
    internal static class IsExternalInit
    {
    }
}
