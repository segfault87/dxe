use dxe_data::entities::{AudioRecording, Booking, Identity};
use dxe_data::queries::identity::get_group_members;
use dxe_types::IdentityProvider;
use sqlx::SqliteConnection;

use crate::config::TimeZoneConfig;
use crate::models::Error;
use crate::services::messaging::MessagingEvent;
use crate::services::messaging::biztalk::BiztalkSender;

pub async fn send_confirmation(
    biztalk_sender: &Option<BiztalkSender>,
    database: &mut SqliteConnection,
    timezone_config: &TimeZoneConfig,
    booking: &Booking,
) -> Result<(), Error> {
    let start = timezone_config.convert(booking.time_from);
    let end = timezone_config.convert(booking.time_to);

    let time_str = format!(
        "{} - {} ({} 시간)",
        start.format("%Y-%m-%d %H:%M"),
        end.format("%Y-%m-%d %H:%M"),
        (end - start).num_hours()
    );

    let recipients = match &booking.customer {
        Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
        Identity::User(u) => vec![u.clone()],
    };

    if let Some(biztalk_sender) = biztalk_sender {
        let biztalk_recipients: Vec<_> = recipients
            .iter()
            .filter_map(|v| {
                if v.provider == IdentityProvider::Kakao {
                    Some(v.foreign_id.clone())
                } else {
                    None
                }
            })
            .collect();

        biztalk_sender.send(MessagingEvent::BookingConfirmation {
            recipients: biztalk_recipients,
            booking_id: booking.id,
            customer_name: booking.customer.name().to_owned(),
            reservation_time: time_str,
        });
    }

    Ok(())
}

pub async fn send_cancellation(
    biztalk_sender: &Option<BiztalkSender>,
    database: &mut SqliteConnection,
    timezone_config: &TimeZoneConfig,
    booking: &Booking,
    refund_rate: i32,
) -> Result<(), Error> {
    let start = timezone_config.convert(booking.time_from);
    let end = timezone_config.convert(booking.time_to);

    let time_str = format!(
        "{} - {} ({} 시간)",
        start.format("%Y-%m-%d %H:%M"),
        end.format("%Y-%m-%d %H:%M"),
        (end - start).num_hours()
    );

    let recipients = match &booking.customer {
        Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
        Identity::User(u) => vec![u.clone()],
    };

    if let Some(biztalk_sender) = biztalk_sender {
        let biztalk_recipients: Vec<_> = recipients
            .iter()
            .filter_map(|v| {
                if v.provider == IdentityProvider::Kakao {
                    Some(v.foreign_id.clone())
                } else {
                    None
                }
            })
            .collect();

        biztalk_sender.send(MessagingEvent::CancelNotification {
            recipients: biztalk_recipients,
            booking_id: booking.id,
            customer_name: booking.customer.name().to_owned(),
            reservation_time: time_str,
            refund_rate,
        });
    }

    Ok(())
}

pub fn send_refund_confirmation(
    biztalk_sender: &Option<BiztalkSender>,
    timezone_config: &TimeZoneConfig,
    booking: &Booking,
    refunded_price: i64,
) {
    let start = timezone_config.convert(booking.time_from);
    let end = timezone_config.convert(booking.time_to);

    let time_str = format!(
        "{} - {} ({} 시간)",
        start.format("%Y-%m-%d %H:%M"),
        end.format("%Y-%m-%d %H:%M"),
        (end - start).num_hours()
    );

    #[allow(clippy::single_match)]
    match booking.holder.provider {
        IdentityProvider::Kakao => {
            if let Some(biztalk_sender) = biztalk_sender {
                biztalk_sender.send(MessagingEvent::RefundNotification {
                    recipient: booking.holder.foreign_id.clone(),
                    booking_id: booking.id,
                    customer_name: booking.customer.name().to_owned(),
                    reservation_time: time_str,
                    refunded_price,
                });
            }
        }
        _ => {}
    }
}

pub async fn send_audio_recording(
    biztalk_sender: &Option<BiztalkSender>,
    database: &mut SqliteConnection,
    timezone_config: &TimeZoneConfig,
    booking: &Booking,
    audio_recording: &AudioRecording,
) -> Result<(), Error> {
    let start = timezone_config.convert(booking.time_from);
    let end = timezone_config.convert(booking.time_to);

    let time_str = format!(
        "{} - {} ({} 시간)",
        start.format("%Y-%m-%d %H:%M"),
        end.format("%Y-%m-%d %H:%M"),
        (end - start).num_hours()
    );

    let expires_in = audio_recording
        .expires_in
        .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string());

    let recipients = match &booking.customer {
        Identity::Group(g) => get_group_members(&mut *database, &g.id).await?,
        Identity::User(u) => vec![u.clone()],
    };

    if let Some(biztalk_sender) = biztalk_sender {
        let biztalk_recipients: Vec<_> = recipients
            .iter()
            .filter_map(|v| {
                if v.provider == IdentityProvider::Kakao {
                    Some(v.foreign_id.clone())
                } else {
                    None
                }
            })
            .collect();

        biztalk_sender.send(MessagingEvent::AudioRecording {
            recipients: biztalk_recipients,
            booking_id: booking.id,
            customer_name: booking.customer.name().to_owned(),
            reservation_time: time_str,
            expires_in,
        });
    }

    Ok(())
}
