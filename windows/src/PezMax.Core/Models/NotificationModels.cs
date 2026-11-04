using PezMax.Core.Json;

namespace PezMax.Core.Models;

public sealed class NotificationItem
{
    public long NotifyId { get; init; }
    public string NotifyType { get; init; } = "";
    public string Title { get; init; } = "";
    public string Content { get; init; } = "";
    public string Status { get; init; } = "";
    public long Sort { get; init; }
    public string DisplayMode { get; init; } = "";
    public string FaultStartTime { get; init; } = "";
    public string FaultEndTime { get; init; } = "";
    public string MaintenanceStartTime { get; init; } = "";
    public string MaintenanceEndTime { get; init; } = "";
    public long RemindBeforeMinutes { get; init; }
    public long UploadUserId { get; init; }
    public long MaterialId { get; init; }
    public string MaterialTitleSnapshot { get; init; } = "";
    public string PublishStart { get; init; } = "";
    public string PublishEnd { get; init; } = "";
    public long ScrollTimeInterval { get; init; }
    public string Remark { get; init; } = "";
    public string CreateTime { get; init; } = "";
    public string UpdateTime { get; init; } = "";

    public string Preview => Content.Length <= 120 ? Content : Content.Substring(0, 120) + "…";
    public string TypeName => NotifyType switch
    {
        "1" => "版本更新",
        "2" => "系统故障",
        "3" => "系统维护",
        "4" => "资料下架",
        "5" => "公告",
        _ => "通知",
    };

    public static NotificationItem From(JsonValue? value) => new()
    {
        NotifyId = JsonModel.Long(value, "notifyId"),
        NotifyType = JsonModel.Str(value, "notifyType"),
        Title = JsonModel.Str(value, "title"),
        Content = JsonModel.Str(value, "content"),
        Status = JsonModel.Str(value, "status"),
        Sort = JsonModel.Long(value, "sort"),
        DisplayMode = JsonModel.Str(value, "displayMode"),
        FaultStartTime = JsonModel.Str(value, "faultStartTime"),
        FaultEndTime = JsonModel.Str(value, "faultEndTime"),
        MaintenanceStartTime = JsonModel.Str(value, "maintenanceStartTime"),
        MaintenanceEndTime = JsonModel.Str(value, "maintenanceEndTime"),
        RemindBeforeMinutes = JsonModel.Long(value, "remindBeforeMinutes"),
        UploadUserId = JsonModel.Long(value, "uploadUserId"),
        MaterialId = JsonModel.Long(value, "materialId"),
        MaterialTitleSnapshot = JsonModel.Str(value, "materialTitleSnapshot"),
        PublishStart = JsonModel.Str(value, "publishStart"),
        PublishEnd = JsonModel.Str(value, "publishEnd"),
        ScrollTimeInterval = JsonModel.Long(value, "scrollTimeInterval"),
        Remark = JsonModel.Str(value, "remark"),
        CreateTime = JsonModel.Str(value, "createTime"),
        UpdateTime = JsonModel.Str(value, "updateTime"),
    };
}
