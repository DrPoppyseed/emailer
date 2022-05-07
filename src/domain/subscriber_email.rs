use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
  pub fn parse(s: String) -> Result<SubscriberEmail, String> {
    if validate_email(&s) {
      Ok(Self(s))
    } else {
      Err(format!("{} is not a valid email.", s))
    }
  }
}

impl AsRef<str> for SubscriberEmail {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use crate::domain::SubscriberEmail;
  use fake::faker::internet::en::SafeEmail;
  use fake::Fake;

  #[derive(Debug, Clone)]
  struct ValidEmailFixture(pub String);

  impl quickcheck::Arbitrary for ValidEmailFixture {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
      let email = SafeEmail().fake_with_rng(g);
      Self(email)
    }
  }

  #[quickcheck_macros::quickcheck]
  fn valid_emails_are_parsed_successfully(
    valid_email: ValidEmailFixture,
  ) -> bool {
    SubscriberEmail::parse(valid_email.0).is_ok()
  }
}
