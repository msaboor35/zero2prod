use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;

pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
