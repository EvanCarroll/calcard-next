/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::jscalendar::*;
use jmap_tools::{JsonPointer, JsonPointerItem, Key};
use std::str::FromStr;

impl<I: JSCalendarId> FromStr for JSCalendarProperty<I> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "@type" => Some(JSCalendarProperty::Type),
            "acknowledged" => Some(JSCalendarProperty::Acknowledged),
            "action" => Some(JSCalendarProperty::Action),
            "alerts" => Some(JSCalendarProperty::Alerts),
            "baseEventId" => Some(JSCalendarProperty::BaseEventId),
            "byDay" => Some(JSCalendarProperty::ByDay),
            "byHour" => Some(JSCalendarProperty::ByHour),
            "byMinute" => Some(JSCalendarProperty::ByMinute),
            "byMonth" => Some(JSCalendarProperty::ByMonth),
            "byMonthDay" => Some(JSCalendarProperty::ByMonthDay),
            "bySecond" => Some(JSCalendarProperty::BySecond),
            "bySetPosition" => Some(JSCalendarProperty::BySetPosition),
            "byWeekNo" => Some(JSCalendarProperty::ByWeekNo),
            "byYearDay" => Some(JSCalendarProperty::ByYearDay),
            "calendarAddress" => Some(JSCalendarProperty::CalendarAddress),
            "calendarIds" => Some(JSCalendarProperty::CalendarIds),
            "categories" => Some(JSCalendarProperty::Categories),
            "color" => Some(JSCalendarProperty::Color),
            "contentType" => Some(JSCalendarProperty::ContentType),
            "coordinates" => Some(JSCalendarProperty::Coordinates),
            "count" => Some(JSCalendarProperty::Count),
            "created" => Some(JSCalendarProperty::Created),
            "day" => Some(JSCalendarProperty::Day),
            "delegatedFrom" => Some(JSCalendarProperty::DelegatedFrom),
            "delegatedTo" => Some(JSCalendarProperty::DelegatedTo),
            "description" => Some(JSCalendarProperty::Description),
            "descriptionContentType" => Some(JSCalendarProperty::DescriptionContentType),
            "display" => Some(JSCalendarProperty::Display),
            "due" => Some(JSCalendarProperty::Due),
            "duration" => Some(JSCalendarProperty::Duration),
            "email" => Some(JSCalendarProperty::Email),
            "entries" => Some(JSCalendarProperty::Entries),
            "estimatedDuration" => Some(JSCalendarProperty::EstimatedDuration),
            "excluded" => Some(JSCalendarProperty::Excluded),
            "expectReply" => Some(JSCalendarProperty::ExpectReply),
            "features" => Some(JSCalendarProperty::Features),
            "firstDayOfWeek" => Some(JSCalendarProperty::FirstDayOfWeek),
            "freeBusyStatus" => Some(JSCalendarProperty::FreeBusyStatus),
            "frequency" => Some(JSCalendarProperty::Frequency),
            "hideAttendees" => Some(JSCalendarProperty::HideAttendees),
            "href" => Some(JSCalendarProperty::Href),
            "id" => Some(JSCalendarProperty::Id),
            "interval" => Some(JSCalendarProperty::Interval),
            "invitedBy" => Some(JSCalendarProperty::InvitedBy),
            "isDraft" => Some(JSCalendarProperty::IsDraft),
            "isOrigin" => Some(JSCalendarProperty::IsOrigin),
            "keywords" => Some(JSCalendarProperty::Keywords),
            "kind" => Some(JSCalendarProperty::Kind),
            "links" => Some(JSCalendarProperty::Links),
            "locale" => Some(JSCalendarProperty::Locale),
            "locations" => Some(JSCalendarProperty::Locations),
            "locationTypes" => Some(JSCalendarProperty::LocationTypes),
            "mayInviteOthers" => Some(JSCalendarProperty::MayInviteOthers),
            "mayInviteSelf" => Some(JSCalendarProperty::MayInviteSelf),
            "memberOf" => Some(JSCalendarProperty::MemberOf),
            "method" => Some(JSCalendarProperty::Method),
            "name" => Some(JSCalendarProperty::Name),
            "nthOfPeriod" => Some(JSCalendarProperty::NthOfPeriod),
            "offset" => Some(JSCalendarProperty::Offset),
            "participants" => Some(JSCalendarProperty::Participants),
            "participationComment" => Some(JSCalendarProperty::ParticipationComment),
            "participationStatus" => Some(JSCalendarProperty::ParticipationStatus),
            "percentComplete" => Some(JSCalendarProperty::PercentComplete),
            "priority" => Some(JSCalendarProperty::Priority),
            "privacy" => Some(JSCalendarProperty::Privacy),
            "prodId" => Some(JSCalendarProperty::ProdId),
            "progress" => Some(JSCalendarProperty::Progress),
            "recurrenceId" => Some(JSCalendarProperty::RecurrenceId),
            "recurrenceIdTimeZone" => Some(JSCalendarProperty::RecurrenceIdTimeZone),
            "recurrenceOverrides" => Some(JSCalendarProperty::RecurrenceOverrides),
            "rel" => Some(JSCalendarProperty::Rel),
            "relatedTo" => Some(JSCalendarProperty::RelatedTo),
            "relation" => Some(JSCalendarProperty::Relation),
            "relativeTo" => Some(JSCalendarProperty::RelativeTo),
            "replyTo" => Some(JSCalendarProperty::ReplyTo),
            "requestStatus" => Some(JSCalendarProperty::RequestStatus),
            "roles" => Some(JSCalendarProperty::Roles),
            "rscale" => Some(JSCalendarProperty::Rscale),
            "sentBy" => Some(JSCalendarProperty::SentBy),
            "scheduleAgent" => Some(JSCalendarProperty::ScheduleAgent),
            "scheduleForceSend" => Some(JSCalendarProperty::ScheduleForceSend),
            "scheduleSequence" => Some(JSCalendarProperty::ScheduleSequence),
            "scheduleStatus" => Some(JSCalendarProperty::ScheduleStatus),
            "scheduleUpdated" => Some(JSCalendarProperty::ScheduleUpdated),
            "sendTo" => Some(JSCalendarProperty::SendTo),
            "sequence" => Some(JSCalendarProperty::Sequence),
            "showWithoutTime" => Some(JSCalendarProperty::ShowWithoutTime),
            "size" => Some(JSCalendarProperty::Size),
            "skip" => Some(JSCalendarProperty::Skip),
            "source" => Some(JSCalendarProperty::Source),
            "start" => Some(JSCalendarProperty::Start),
            "status" => Some(JSCalendarProperty::Status),
            "timeZone" => Some(JSCalendarProperty::TimeZone),
            "title" => Some(JSCalendarProperty::Title),
            "trigger" => Some(JSCalendarProperty::Trigger),
            "uid" => Some(JSCalendarProperty::Uid),
            "until" => Some(JSCalendarProperty::Until),
            "updated" => Some(JSCalendarProperty::Updated),
            "uri" => Some(JSCalendarProperty::Uri),
            "useDefaultAlerts" => Some(JSCalendarProperty::UseDefaultAlerts),
            "utcEnd" => Some(JSCalendarProperty::UtcEnd),
            "utcStart" => Some(JSCalendarProperty::UtcStart),
            "version" => Some(JSCalendarProperty::Version),
            "virtualLocations" => Some(JSCalendarProperty::VirtualLocations),
            "when" => Some(JSCalendarProperty::When),
            "endTimeZone" => Some(JSCalendarProperty::EndTimeZone),
            "mainLocationId" => Some(JSCalendarProperty::MainLocationId),
            "organizerCalendarAddress" => Some(JSCalendarProperty::OrganizerCalendarAddress),
            "recurrenceRule" => Some(JSCalendarProperty::RecurrenceRule),
            "properties" => Some(JSCalendarProperty::Properties),
            "components" => Some(JSCalendarProperty::Components),
            "valueType" => Some(JSCalendarProperty::ValueType),
            "convertedProperties" => Some(JSCalendarProperty::ConvertedProperties),
            "parameters" => Some(JSCalendarProperty::Parameters),
            "iCalendar" => Some(JSCalendarProperty::ICalendar),
            "blobId" => Some(JSCalendarProperty::BlobId),
            _ => None,
        )
        .ok_or(())
    }
}

impl<I: JSCalendarId> JSCalendarProperty<I> {
    pub fn to_string(&self) -> Cow<'static, str> {
        match self {
            JSCalendarProperty::Type => "@type",
            JSCalendarProperty::Acknowledged => "acknowledged",
            JSCalendarProperty::Action => "action",
            JSCalendarProperty::Alerts => "alerts",
            JSCalendarProperty::BaseEventId => "baseEventId",
            JSCalendarProperty::ByDay => "byDay",
            JSCalendarProperty::ByHour => "byHour",
            JSCalendarProperty::ByMinute => "byMinute",
            JSCalendarProperty::ByMonth => "byMonth",
            JSCalendarProperty::ByMonthDay => "byMonthDay",
            JSCalendarProperty::BySecond => "bySecond",
            JSCalendarProperty::BySetPosition => "bySetPosition",
            JSCalendarProperty::ByWeekNo => "byWeekNo",
            JSCalendarProperty::ByYearDay => "byYearDay",
            JSCalendarProperty::CalendarAddress => "calendarAddress",
            JSCalendarProperty::CalendarIds => "calendarIds",
            JSCalendarProperty::Categories => "categories",
            JSCalendarProperty::Color => "color",
            JSCalendarProperty::ContentType => "contentType",
            JSCalendarProperty::Coordinates => "coordinates",
            JSCalendarProperty::Count => "count",
            JSCalendarProperty::Created => "created",
            JSCalendarProperty::Day => "day",
            JSCalendarProperty::DelegatedFrom => "delegatedFrom",
            JSCalendarProperty::DelegatedTo => "delegatedTo",
            JSCalendarProperty::Description => "description",
            JSCalendarProperty::DescriptionContentType => "descriptionContentType",
            JSCalendarProperty::Display => "display",
            JSCalendarProperty::Due => "due",
            JSCalendarProperty::Duration => "duration",
            JSCalendarProperty::Email => "email",
            JSCalendarProperty::Entries => "entries",
            JSCalendarProperty::EstimatedDuration => "estimatedDuration",
            JSCalendarProperty::Excluded => "excluded",
            JSCalendarProperty::ExpectReply => "expectReply",
            JSCalendarProperty::Features => "features",
            JSCalendarProperty::FirstDayOfWeek => "firstDayOfWeek",
            JSCalendarProperty::FreeBusyStatus => "freeBusyStatus",
            JSCalendarProperty::Frequency => "frequency",
            JSCalendarProperty::HideAttendees => "hideAttendees",
            JSCalendarProperty::Href => "href",
            JSCalendarProperty::Id => "id",
            JSCalendarProperty::Interval => "interval",
            JSCalendarProperty::InvitedBy => "invitedBy",
            JSCalendarProperty::IsDraft => "isDraft",
            JSCalendarProperty::IsOrigin => "isOrigin",
            JSCalendarProperty::Keywords => "keywords",
            JSCalendarProperty::Kind => "kind",
            JSCalendarProperty::Links => "links",
            JSCalendarProperty::Locale => "locale",
            JSCalendarProperty::Locations => "locations",
            JSCalendarProperty::LocationTypes => "locationTypes",
            JSCalendarProperty::MayInviteOthers => "mayInviteOthers",
            JSCalendarProperty::MayInviteSelf => "mayInviteSelf",
            JSCalendarProperty::MemberOf => "memberOf",
            JSCalendarProperty::Method => "method",
            JSCalendarProperty::Name => "name",
            JSCalendarProperty::NthOfPeriod => "nthOfPeriod",
            JSCalendarProperty::Offset => "offset",
            JSCalendarProperty::Participants => "participants",
            JSCalendarProperty::ParticipationComment => "participationComment",
            JSCalendarProperty::ParticipationStatus => "participationStatus",
            JSCalendarProperty::PercentComplete => "percentComplete",
            JSCalendarProperty::Priority => "priority",
            JSCalendarProperty::Privacy => "privacy",
            JSCalendarProperty::ProdId => "prodId",
            JSCalendarProperty::Progress => "progress",
            JSCalendarProperty::RecurrenceId => "recurrenceId",
            JSCalendarProperty::RecurrenceIdTimeZone => "recurrenceIdTimeZone",
            JSCalendarProperty::RecurrenceOverrides => "recurrenceOverrides",
            JSCalendarProperty::Rel => "rel",
            JSCalendarProperty::RelatedTo => "relatedTo",
            JSCalendarProperty::Relation => "relation",
            JSCalendarProperty::RelativeTo => "relativeTo",
            JSCalendarProperty::ReplyTo => "replyTo",
            JSCalendarProperty::RequestStatus => "requestStatus",
            JSCalendarProperty::Roles => "roles",
            JSCalendarProperty::Rscale => "rscale",
            JSCalendarProperty::SentBy => "sentBy",
            JSCalendarProperty::ScheduleAgent => "scheduleAgent",
            JSCalendarProperty::ScheduleForceSend => "scheduleForceSend",
            JSCalendarProperty::ScheduleSequence => "scheduleSequence",
            JSCalendarProperty::ScheduleStatus => "scheduleStatus",
            JSCalendarProperty::ScheduleUpdated => "scheduleUpdated",
            JSCalendarProperty::SendTo => "sendTo",
            JSCalendarProperty::Sequence => "sequence",
            JSCalendarProperty::ShowWithoutTime => "showWithoutTime",
            JSCalendarProperty::Size => "size",
            JSCalendarProperty::Skip => "skip",
            JSCalendarProperty::Source => "source",
            JSCalendarProperty::Start => "start",
            JSCalendarProperty::Status => "status",
            JSCalendarProperty::TimeZone => "timeZone",
            JSCalendarProperty::Title => "title",
            JSCalendarProperty::Trigger => "trigger",
            JSCalendarProperty::Uid => "uid",
            JSCalendarProperty::Until => "until",
            JSCalendarProperty::Updated => "updated",
            JSCalendarProperty::Uri => "uri",
            JSCalendarProperty::UseDefaultAlerts => "useDefaultAlerts",
            JSCalendarProperty::UtcEnd => "utcEnd",
            JSCalendarProperty::UtcStart => "utcStart",
            JSCalendarProperty::Version => "version",
            JSCalendarProperty::VirtualLocations => "virtualLocations",
            JSCalendarProperty::When => "when",
            JSCalendarProperty::EndTimeZone => "endTimeZone",
            JSCalendarProperty::MainLocationId => "mainLocationId",
            JSCalendarProperty::OrganizerCalendarAddress => "organizerCalendarAddress",
            JSCalendarProperty::RecurrenceRule => "recurrenceRule",
            JSCalendarProperty::Properties => "properties",
            JSCalendarProperty::Components => "components",
            JSCalendarProperty::ValueType => "valueType",
            JSCalendarProperty::ConvertedProperties => "convertedProperties",
            JSCalendarProperty::Parameters => "parameters",
            JSCalendarProperty::ICalendar => "iCalendar",
            JSCalendarProperty::BlobId => "blobId",
            JSCalendarProperty::LinkDisplay(v) => v.as_str(),
            JSCalendarProperty::VirtualLocationFeature(v) => v.as_str(),
            JSCalendarProperty::ParticipantRole(v) => v.as_str(),
            JSCalendarProperty::RelationValue(v) => v.as_str(),
            JSCalendarProperty::LinkRelation(v) => return v.as_str().to_string().into(),
            JSCalendarProperty::DateTime(dt) => return dt.to_rfc3339().into(),
            JSCalendarProperty::Pointer(pointer) => return Cow::Owned(pointer.to_string()),
            JSCalendarProperty::IdValue(id) => return id.to_string().into(),
            JSCalendarProperty::IdReference(s) => return format!("#{}", s).into(),
        }
        .into()
    }

    pub(crate) fn is_forbidden_override_patch(&self) -> bool {
        matches!(
            self,
            JSCalendarProperty::Type
                | JSCalendarProperty::Method
                | JSCalendarProperty::OrganizerCalendarAddress
                | JSCalendarProperty::Privacy
                | JSCalendarProperty::ProdId
                | JSCalendarProperty::RecurrenceId
                | JSCalendarProperty::RecurrenceIdTimeZone
                | JSCalendarProperty::SentBy
                | JSCalendarProperty::Uid
                | JSCalendarProperty::RecurrenceOverrides
                | JSCalendarProperty::RecurrenceRule
        )
    }

    pub(crate) fn is_forbidden_override_pointer(pointer: &JsonPointer<Self>) -> bool {
        let mut items = pointer
            .iter()
            .filter(|item| !matches!(item, JsonPointerItem::Root));

        match (items.next(), items.next(), items.next(), items.next()) {
            (
                Some(JsonPointerItem::Key(Key::Property(
                    JSCalendarProperty::RecurrenceOverrides | JSCalendarProperty::RecurrenceRule,
                ))),
                ..,
            ) => true,
            (Some(JsonPointerItem::Key(Key::Property(property))), None, ..) => {
                property.is_forbidden_override_patch()
            }
            (
                Some(JsonPointerItem::Key(Key::Property(JSCalendarProperty::Participants))),
                Some(_),
                Some(JsonPointerItem::Key(Key::Property(JSCalendarProperty::CalendarAddress))),
                None,
            ) => true,
            _ => false,
        }
    }
}

impl FromStr for JSCalendarType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "Event" => Some(JSCalendarType::Event),
            "Task" => Some(JSCalendarType::Task),
            "Group" => Some(JSCalendarType::Group),
            "Alert" => Some(JSCalendarType::Alert),
            "Boolean" => Some(JSCalendarType::Boolean),
            "Duration" => Some(JSCalendarType::Duration),
            "Id" => Some(JSCalendarType::Id),
            "Int" => Some(JSCalendarType::Int),
            "LocalDateTime" => Some(JSCalendarType::LocalDateTime),
            "Link" => Some(JSCalendarType::Link),
            "Location" => Some(JSCalendarType::Location),
            "NDay" => Some(JSCalendarType::NDay),
            "Number" => Some(JSCalendarType::Number),
            "Participant" => Some(JSCalendarType::Participant),
            "PatchObject" => Some(JSCalendarType::PatchObject),
            "RecurrenceRule" => Some(JSCalendarType::RecurrenceRule),
            "Relation" => Some(JSCalendarType::Relation),
            "SignedDuration" => Some(JSCalendarType::SignedDuration),
            "String" => Some(JSCalendarType::String),
            "TimeZone" => Some(JSCalendarType::TimeZone),
            "TimeZoneId" => Some(JSCalendarType::TimeZoneId),
            "TimeZoneRule" => Some(JSCalendarType::TimeZoneRule),
            "UnsignedInt" => Some(JSCalendarType::UnsignedInt),
            "UTCDateTime" => Some(JSCalendarType::UTCDateTime),
            "VirtualLocation" => Some(JSCalendarType::VirtualLocation),
            "ICalComponent" => Some(JSCalendarType::ICalComponent),
            "AbsoluteTrigger" => Some(JSCalendarType::AbsoluteTrigger),
            "OffsetTrigger" => Some(JSCalendarType::OffsetTrigger),
            _ => None,
        )
        .ok_or(())
    }
}

impl JSCalendarType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarType::Event => "Event",
            JSCalendarType::Task => "Task",
            JSCalendarType::Group => "Group",
            JSCalendarType::Alert => "Alert",
            JSCalendarType::Boolean => "Boolean",
            JSCalendarType::Duration => "Duration",
            JSCalendarType::Id => "Id",
            JSCalendarType::Int => "Int",
            JSCalendarType::LocalDateTime => "LocalDateTime",
            JSCalendarType::Link => "Link",
            JSCalendarType::Location => "Location",
            JSCalendarType::NDay => "NDay",
            JSCalendarType::Number => "Number",
            JSCalendarType::Participant => "Participant",
            JSCalendarType::PatchObject => "PatchObject",
            JSCalendarType::RecurrenceRule => "RecurrenceRule",
            JSCalendarType::Relation => "Relation",
            JSCalendarType::SignedDuration => "SignedDuration",
            JSCalendarType::String => "String",
            JSCalendarType::TimeZone => "TimeZone",
            JSCalendarType::TimeZoneId => "TimeZoneId",
            JSCalendarType::TimeZoneRule => "TimeZoneRule",
            JSCalendarType::UnsignedInt => "UnsignedInt",
            JSCalendarType::UTCDateTime => "UTCDateTime",
            JSCalendarType::VirtualLocation => "VirtualLocation",
            JSCalendarType::ICalComponent => "ICalComponent",
            JSCalendarType::OffsetTrigger => "OffsetTrigger",
            JSCalendarType::AbsoluteTrigger => "AbsoluteTrigger",
        }
    }
}

impl FromStr for JSCalendarAlertAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "display" => Some(JSCalendarAlertAction::Display),
            "email" => Some(JSCalendarAlertAction::Email)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarAlertAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarAlertAction::Display => "display",
            JSCalendarAlertAction::Email => "email",
        }
    }
}

impl FromStr for JSCalendarLinkDisplay {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "badge" => Some(JSCalendarLinkDisplay::Badge),
            "graphic" => Some(JSCalendarLinkDisplay::Graphic),
            "fullsize" => Some(JSCalendarLinkDisplay::Fullsize),
            "thumbnail" => Some(JSCalendarLinkDisplay::Thumbnail)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarLinkDisplay {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarLinkDisplay::Badge => "badge",
            JSCalendarLinkDisplay::Graphic => "graphic",
            JSCalendarLinkDisplay::Fullsize => "fullsize",
            JSCalendarLinkDisplay::Thumbnail => "thumbnail",
        }
    }
}

impl FromStr for JSCalendarVirtualLocationFeature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "audio" => Some(JSCalendarVirtualLocationFeature::Audio),
            "chat" => Some(JSCalendarVirtualLocationFeature::Chat),
            "feed" => Some(JSCalendarVirtualLocationFeature::Feed),
            "moderator" => Some(JSCalendarVirtualLocationFeature::Moderator),
            "phone" => Some(JSCalendarVirtualLocationFeature::Phone),
            "screen" => Some(JSCalendarVirtualLocationFeature::Screen),
            "video" => Some(JSCalendarVirtualLocationFeature::Video)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarVirtualLocationFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarVirtualLocationFeature::Audio => "audio",
            JSCalendarVirtualLocationFeature::Chat => "chat",
            JSCalendarVirtualLocationFeature::Feed => "feed",
            JSCalendarVirtualLocationFeature::Moderator => "moderator",
            JSCalendarVirtualLocationFeature::Phone => "phone",
            JSCalendarVirtualLocationFeature::Screen => "screen",
            JSCalendarVirtualLocationFeature::Video => "video",
        }
    }
}

impl FromStr for JSCalendarFreeBusyStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "free" => Some(JSCalendarFreeBusyStatus::Free),
            "busy" => Some(JSCalendarFreeBusyStatus::Busy)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarFreeBusyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarFreeBusyStatus::Free => "free",
            JSCalendarFreeBusyStatus::Busy => "busy",
        }
    }
}

impl FromStr for JSCalendarParticipantKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "individual" => Some(JSCalendarParticipantKind::Individual),
            "group" => Some(JSCalendarParticipantKind::Group),
            "resource" => Some(JSCalendarParticipantKind::Resource),
            "location" => Some(JSCalendarParticipantKind::Location)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarParticipantKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipantKind::Individual => "individual",
            JSCalendarParticipantKind::Group => "group",
            JSCalendarParticipantKind::Resource => "resource",
            JSCalendarParticipantKind::Location => "location",
        }
    }
}

impl FromStr for JSCalendarParticipationStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "needs-action" => Some(JSCalendarParticipationStatus::NeedsAction),
            "accepted" => Some(JSCalendarParticipationStatus::Accepted),
            "declined" => Some(JSCalendarParticipationStatus::Declined),
            "tentative" => Some(JSCalendarParticipationStatus::Tentative),
            "delegated" => Some(JSCalendarParticipationStatus::Delegated)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarParticipationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipationStatus::NeedsAction => "needs-action",
            JSCalendarParticipationStatus::Accepted => "accepted",
            JSCalendarParticipationStatus::Declined => "declined",
            JSCalendarParticipationStatus::Tentative => "tentative",
            JSCalendarParticipationStatus::Delegated => "delegated",
        }
    }
}

impl FromStr for JSCalendarPrivacy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "public" => Some(JSCalendarPrivacy::Public),
            "private" => Some(JSCalendarPrivacy::Private),
            "secret" => Some(JSCalendarPrivacy::Secret)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarPrivacy {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarPrivacy::Public => "public",
            JSCalendarPrivacy::Private => "private",
            JSCalendarPrivacy::Secret => "secret",
        }
    }
}

impl FromStr for JSCalendarProgress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "needs-action" => Some(JSCalendarProgress::NeedsAction),
            "in-process" => Some(JSCalendarProgress::InProcess),
            "completed" => Some(JSCalendarProgress::Completed),
            "failed" => Some(JSCalendarProgress::Failed),
            "cancelled" => Some(JSCalendarProgress::Cancelled),
            _ => None,
        )
        .ok_or(())
    }
}

impl JSCalendarProgress {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarProgress::NeedsAction => "needs-action",
            JSCalendarProgress::InProcess => "in-process",
            JSCalendarProgress::Completed => "completed",
            JSCalendarProgress::Failed => "failed",
            JSCalendarProgress::Cancelled => "cancelled",
        }
    }
}

impl FromStr for JSCalendarRelation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "first" => Some(JSCalendarRelation::First),
            "next" => Some(JSCalendarRelation::Next),
            "child" => Some(JSCalendarRelation::Child),
            "parent" => Some(JSCalendarRelation::Parent),
            "snooze" => Some(JSCalendarRelation::Snooze),
            _ => None,
        )
        .ok_or(())
    }
}

impl JSCalendarRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarRelation::First => "first",
            JSCalendarRelation::Next => "next",
            JSCalendarRelation::Child => "child",
            JSCalendarRelation::Parent => "parent",
            JSCalendarRelation::Snooze => "snooze",
        }
    }
}

impl FromStr for JSCalendarRelativeTo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "start" => Some(JSCalendarRelativeTo::Start),
            "end" => Some(JSCalendarRelativeTo::End)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarRelativeTo {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarRelativeTo::Start => "start",
            JSCalendarRelativeTo::End => "end",
        }
    }
}

impl FromStr for JSCalendarParticipantRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "owner" => Some(JSCalendarParticipantRole::Owner),
            "optional" => Some(JSCalendarParticipantRole::Optional),
            "informational" => Some(JSCalendarParticipantRole::Informational),
            "chair" => Some(JSCalendarParticipantRole::Chair),
            "required" => Some(JSCalendarParticipantRole::Required),
            _ => None,
        )
        .ok_or(())
    }
}

impl JSCalendarParticipantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipantRole::Owner => "owner",
            JSCalendarParticipantRole::Optional => "optional",
            JSCalendarParticipantRole::Informational => "informational",
            JSCalendarParticipantRole::Chair => "chair",
            JSCalendarParticipantRole::Required => "required",
        }
    }
}

impl FromStr for JSCalendarScheduleAgent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "server" => Some(JSCalendarScheduleAgent::Server),
            "client" => Some(JSCalendarScheduleAgent::Client),
            "none" => Some(JSCalendarScheduleAgent::None)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarScheduleAgent {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarScheduleAgent::Server => "server",
            JSCalendarScheduleAgent::Client => "client",
            JSCalendarScheduleAgent::None => "none",
        }
    }
}

impl FromStr for JSCalendarEventStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::fnc_map!(s.as_bytes(),
            "confirmed" => Some(JSCalendarEventStatus::Confirmed),
            "cancelled" => Some(JSCalendarEventStatus::Cancelled),
            "tentative" => Some(JSCalendarEventStatus::Tentative)
        ,
            _ => None,)
        .ok_or(())
    }
}

impl JSCalendarEventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarEventStatus::Confirmed => "confirmed",
            JSCalendarEventStatus::Cancelled => "cancelled",
            JSCalendarEventStatus::Tentative => "tentative",
        }
    }
}

impl ICalendarFrequency {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarFrequency::Secondly => "secondly",
            ICalendarFrequency::Minutely => "minutely",
            ICalendarFrequency::Hourly => "hourly",
            ICalendarFrequency::Daily => "daily",
            ICalendarFrequency::Weekly => "weekly",
            ICalendarFrequency::Monthly => "monthly",
            ICalendarFrequency::Yearly => "yearly",
        }
    }
}

impl ICalendarSkip {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarSkip::Omit => "omit",
            ICalendarSkip::Backward => "backward",
            ICalendarSkip::Forward => "forward",
        }
    }
}

impl ICalendarWeekday {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarWeekday::Sunday => "su",
            ICalendarWeekday::Monday => "mo",
            ICalendarWeekday::Tuesday => "tu",
            ICalendarWeekday::Wednesday => "we",
            ICalendarWeekday::Thursday => "th",
            ICalendarWeekday::Friday => "fr",
            ICalendarWeekday::Saturday => "sa",
        }
    }
}

impl ICalendarMethod {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarMethod::Publish => "publish",
            ICalendarMethod::Request => "request",
            ICalendarMethod::Reply => "reply",
            ICalendarMethod::Add => "add",
            ICalendarMethod::Cancel => "cancel",
            ICalendarMethod::Refresh => "refresh",
            ICalendarMethod::Counter => "counter",
            ICalendarMethod::Declinecounter => "declinecounter",
        }
    }
}
