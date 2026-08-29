//! Shared lifecycle and classification enums.
//!
//! Statuses are stored in PostgreSQL as `TEXT` columns with CHECK
//! constraints; the string form is the snake_case variant name. Money is
//! always expressed in minor units (paise) with an explicit currency.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! text_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($text => Some($name::$variant),)+
                        _ => None,
                }
            }

            pub const fn all() -> &'static [Self] {
                &[$($name::$variant),+]
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

text_enum!(JobStatus {
    Created => "created",
    Analyzing => "analyzing",
    MechanicsSearching => "mechanics_searching",
    MechanicsNotified => "mechanics_notified",
    OffersReceived => "offers_received",
    MechanicSelected => "mechanic_selected",
    MechanicEnRoute => "mechanic_en_route",
    MechanicArrived => "mechanic_arrived",
    RepairInProgress => "repair_in_progress",
    RepairCompleted => "repair_completed",
    PaymentPending => "payment_pending",
    Completed => "completed",
    Cancelled => "cancelled",
    Expired => "expired",
    Failed => "failed",
    NoMechanicAvailable => "no_mechanic_available",
});

text_enum!(UserRole {
    Customer => "customer",
    Mechanic => "mechanic",
    Admin => "admin",
});

text_enum!(UserStatus {
    Active => "active",
    Suspended => "suspended",
    Deleted => "deleted",
});

text_enum!(OnboardingStatus {
    Pending => "pending",
    Complete => "complete",
});

text_enum!(VehicleKind {
    Motorcycle => "motorcycle",
    Scooter => "scooter",
    Car => "car",
    Van => "van",
    Truck => "truck",
    Other => "other",
});

text_enum!(RepairCategory {
    Battery => "battery",
    Tyre => "tyre",
    Engine => "engine",
    Electrical => "electrical",
    Brakes => "brakes",
    Clutch => "clutch",
    Fuel => "fuel",
    ChainDrive => "chain_drive",
    Cooling => "cooling",
    BodyDamage => "body_damage",
    Lockout => "lockout",
    Towing => "towing",
    Diagnostic => "diagnostic",
    Other => "other",
});

text_enum!(AvailabilityStatus {
    Online => "online",
    Busy => "busy",
    Offline => "offline",
});

text_enum!(OfferStatus {
    Pending => "pending",
    Accepted => "accepted",
    Withdrawn => "withdrawn",
    Expired => "expired",
    Rejected => "rejected",
});

text_enum!(MediaKind {
    Photo => "photo",
    Video => "video",
});

text_enum!(Severity {
    Low => "low",
    Medium => "medium",
    High => "high",
    Critical => "critical",
});

text_enum!(NotificationChannel {
    Push => "push",
    Sms => "sms",
    Email => "email",
    InApp => "in_app",
});

text_enum!(NotificationKind {
    NewBreakdownNearby => "new_breakdown_nearby",
    OfferReceived => "offer_received",
    OfferAccepted => "offer_accepted",
    OfferExpired => "offer_expired",
    MechanicEnRoute => "mechanic_en_route",
    MechanicArrived => "mechanic_arrived",
    JobCompleted => "job_completed",
    JobCancelled => "job_cancelled",
    JobNoMechanic => "job_no_mechanic",
    PaymentReceipt => "payment_receipt",
});

text_enum!(NotificationStatus {
    Pending => "pending",
    Sent => "sent",
    Failed => "failed",
});

text_enum!(PaymentStatus {
    Initiated => "initiated",
    Confirmed => "confirmed",
    Failed => "failed",
    Refunded => "refunded",
});

text_enum!(PaymentMethod {
    Cash => "cash",
    Upi => "upi",
    Card => "card",
});

text_enum!(AgentKind {
    Diagnosis => "diagnosis",
    PriceEstimate => "price_estimate",
});

text_enum!(AgentRunStatus {
    Succeeded => "succeeded",
    Failed => "failed",
});

text_enum!(EstimateSource {
    Agent => "agent",
    Historical => "historical",
});

/// Fixed currency for the initial marketplace (amounts are minor units).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Currency {
    #[default]
    Inr,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        "INR"
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_text_form() {
        for status in JobStatus::all() {
            assert_eq!(JobStatus::parse(status.as_str()), Some(*status));
        }
        assert_eq!(JobStatus::parse("nope"), None);
    }

    #[test]
    fn serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&JobStatus::MechanicsSearching).unwrap(),
            "\"mechanics_searching\""
        );
    }
}
