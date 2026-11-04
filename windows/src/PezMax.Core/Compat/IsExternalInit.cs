namespace System.Runtime.CompilerServices;

// netstandard2.0 没有 init-only 所需的编译器标记；C# 10 的 init 访问器靠它。
internal static class IsExternalInit
{
}
