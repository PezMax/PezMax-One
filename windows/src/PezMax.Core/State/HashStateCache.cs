namespace PezMax.Core.State;

/// <summary>
/// 哈希缓存协议：unchanged=true 时保留本地 payload；否则整体替换。
/// 不负责持久化，调用方把 Hash 与 payload 一起存。
/// </summary>
public sealed class HashStateCache<T>
{
    public string? Hash { get; private set; }
    public T? Payload { get; private set; }
    public bool HasPayload => Payload != null;

    public bool Apply(string? hash, bool unchanged, T? incoming)
    {
        if (unchanged && Payload != null)
        {
            if (!string.IsNullOrEmpty(hash)) Hash = hash;
            return false;
        }

        if (incoming != null)
        {
            Payload = incoming;
            Hash = hash;
            return true;
        }

        if (!unchanged)
        {
            Payload = default;
            Hash = hash;
            return true;
        }

        return false;
    }

    public void Clear()
    {
        Payload = default;
        Hash = null;
    }
}
