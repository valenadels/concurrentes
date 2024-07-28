use serde::Deserialize;

/// Struct to store the container information.
#[derive(Debug, Deserialize, Clone)]
pub struct Container {
    size: usize,
    flavours: Vec<String>,
}

impl Container {
    /// Creates a new Container with the given size and flavours.
    /// # Arguments
    /// - size: The size of the container.
    /// - flavours: The flavours of the container.
    /// # Returns
    /// A new Container.
    pub fn new(size: usize, flavours: Vec<String>) -> Container {
        Self { size, flavours }
    }

    /// Serializes the Container into a byte vector.
    /// # Returns
    /// A byte vector with the serialized Container.
    /// # Errors
    /// Returns a ScreenError if the flavours cannot be serialized.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut container = Vec::new();

        container.extend((self.size as u16).to_be_bytes());
        container.extend((self.flavours.len() as u8).to_be_bytes());

        for flavour in &self.flavours {
            container.push(self.flavour_to_bytes(flavour));
        }

        container
    }

    /// Maps the flavour to the corresponding id.
    /// # Arguments
    /// - flavour: The flavour to map.
    /// # Returns
    /// The id of the flavour.
    /// # Errors
    /// Returns a ScreenError if the flavour is not recognized.
    fn flavour_to_bytes(&self, flavour: &str) -> u8 {
        match flavour {
            "vanilla" => 0,
            "chocolate" => 1,
            "strawberry" => 2,
            "cookies" => 3,
            _ => {
                println!("Flavour unknown with ID {}", flavour);
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_two_flavours() {
        let container = Container::new(250, vec!["cookies".to_string(), "strawberry".to_string()]);
        assert_eq!(container.size, 250);
        assert_eq!(
            container.flavours,
            vec!["cookies".to_string(), "strawberry".to_string()]
        );
    }

    #[test]
    fn test_container_to_bytes() {
        let container = Container::new(
            10000,
            vec![
                "vanilla".to_string(),
                "chocolate".to_string(),
                "cookies".to_string(),
                "strawberry".to_string(),
            ],
        );
        let bytes = container.to_bytes();
        let size_as_bytes = 10000u16.to_be_bytes();
        let flavours_len = 4u8.to_be_bytes();
        assert_eq!(bytes[0..2], size_as_bytes);
        assert_eq!(bytes[2..3], flavours_len);
        assert_eq!(bytes[3], 0);
        assert_eq!(bytes[4], 1);
        assert_eq!(bytes[5], 3);
        assert_eq!(bytes[6], 2);
    }
}
