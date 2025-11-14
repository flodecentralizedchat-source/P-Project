use serde::{Deserialize, Serialize};

/// Price simulation for tokenomics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSimulation {
    year: i32,
    unlocked_percentage: f64,
    unlocked_tokens: f64,
    target_market_cap: f64,
    implied_price: f64,
}

impl PriceSimulation {
    /// Create a new price simulation entry
    pub fn new(
        year: i32,
        unlocked_percentage: f64,
        unlocked_tokens: f64,
        target_market_cap: f64,
        implied_price: f64,
    ) -> Self {
        Self {
            year,
            unlocked_percentage,
            unlocked_tokens,
            target_market_cap,
            implied_price,
        }
    }

    /// Get the year for this simulation
    pub fn get_year(&self) -> i32 {
        self.year
    }

    /// Get the unlocked percentage for this year
    pub fn get_unlocked_percentage(&self) -> f64 {
        self.unlocked_percentage
    }

    /// Get the number of unlocked tokens for this year
    pub fn get_unlocked_tokens(&self) -> f64 {
        self.unlocked_tokens
    }

    /// Get the target market cap for this year
    pub fn get_target_market_cap(&self) -> f64 {
        self.target_market_cap
    }

    /// Get the implied price for this year
    pub fn get_implied_price(&self) -> f64 {
        self.implied_price
    }

    /// Calculate implied price based on market cap and unlocked tokens
    pub fn calculate_implied_price(market_cap: f64, unlocked_tokens: f64) -> f64 {
        if unlocked_tokens <= 0.0 {
            0.0
        } else {
            market_cap / unlocked_tokens
        }
    }

    /// Update the simulation with new parameters
    pub fn update_simulation(
        &mut self,
        unlocked_percentage: f64,
        target_market_cap: f64,
    ) -> f64 {
        self.unlocked_percentage = unlocked_percentage;
        self.target_market_cap = target_market_cap;
        self.implied_price = Self::calculate_implied_price(target_market_cap, self.unlocked_tokens);
        self.implied_price
    }
}

/// Complete price simulation for all years
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletePriceSimulation {
    simulations: Vec<PriceSimulation>,
    total_supply: f64,
}

impl CompletePriceSimulation {
    /// Create a new complete price simulation
    pub fn new(total_supply: f64) -> Self {
        let simulations = vec![
            PriceSimulation::new(1, 25.0, total_supply * 0.25, 14000000.0, 0.16),
            PriceSimulation::new(2, 40.0, total_supply * 0.40, 35000000.0, 0.25),
            PriceSimulation::new(3, 55.0, total_supply * 0.55, 70000000.0, 0.3636),
            PriceSimulation::new(4, 70.0, total_supply * 0.70, 175000000.0, 0.7143),
            PriceSimulation::new(5, 100.0, total_supply * 1.00, 350000000.0, 1.00),
        ];

        Self {
            simulations,
            total_supply,
        }
    }

    /// Get all simulations
    pub fn get_simulations(&self) -> &Vec<PriceSimulation> {
        &self.simulations
    }

    /// Get simulation for a specific year
    pub fn get_simulation_for_year(&self, year: i32) -> Option<&PriceSimulation> {
        self.simulations.iter().find(|sim| sim.get_year() == year)
    }

    /// Calculate custom simulation based on unlocked percentage and market cap
    pub fn calculate_custom_simulation(
        &self,
        unlocked_percentage: f64,
        target_market_cap: f64,
    ) -> PriceSimulation {
        let unlocked_tokens = self.total_supply * (unlocked_percentage / 100.0);
        let implied_price = PriceSimulation::calculate_implied_price(target_market_cap, unlocked_tokens);
        
        PriceSimulation::new(
            0, // Custom simulation
            unlocked_percentage,
            unlocked_tokens,
            target_market_cap,
            implied_price,
        )
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> f64 {
        self.total_supply
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_simulation_creation() {
        let simulation = PriceSimulation::new(1, 25.0, 87500000.0, 14000000.0, 0.16);
        
        assert_eq!(simulation.get_year(), 1);
        assert_eq!(simulation.get_unlocked_percentage(), 25.0);
        assert_eq!(simulation.get_unlocked_tokens(), 87500000.0);
        assert_eq!(simulation.get_target_market_cap(), 14000000.0);
        assert_eq!(simulation.get_implied_price(), 0.16);
    }

    #[test]
    fn test_implied_price_calculation() {
        let implied_price = PriceSimulation::calculate_implied_price(14000000.0, 87500000.0);
        assert_eq!(implied_price, 0.16);
    }

    #[test]
    fn test_simulation_update() {
        let mut simulation = PriceSimulation::new(1, 25.0, 87500000.0, 14000000.0, 0.16);
        let new_price = simulation.update_simulation(30.0, 20000000.0);
        
        assert_eq!(simulation.get_unlocked_percentage(), 30.0);
        assert_eq!(simulation.get_target_market_cap(), 20000000.0);
        // The unlocked tokens should remain the same (87,500,000)
        assert_eq!(simulation.get_unlocked_tokens(), 87500000.0);
        let expected_price = 20000000.0 / 87500000.0;
        assert_eq!(simulation.get_implied_price(), expected_price);
        assert_eq!(new_price, expected_price);
    }

    #[test]
    fn test_complete_price_simulation() {
        let total_supply = 350000000.0;
        let complete_simulation = CompletePriceSimulation::new(total_supply);
        
        assert_eq!(complete_simulation.get_total_supply(), total_supply);
        assert_eq!(complete_simulation.get_simulations().len(), 5);
        
        let year_1_simulation = complete_simulation.get_simulation_for_year(1).unwrap();
        assert_eq!(year_1_simulation.get_unlocked_percentage(), 25.0);
        assert_eq!(year_1_simulation.get_unlocked_tokens(), total_supply * 0.25);
    }

    #[test]
    fn test_custom_simulation() {
        let total_supply = 350000000.0;
        let complete_simulation = CompletePriceSimulation::new(total_supply);
        
        let custom_simulation = complete_simulation.calculate_custom_simulation(30.0, 25000000.0);
        assert_eq!(custom_simulation.get_unlocked_percentage(), 30.0);
        assert_eq!(custom_simulation.get_unlocked_tokens(), total_supply * 0.30);
        assert_eq!(custom_simulation.get_target_market_cap(), 25000000.0);
        assert_eq!(custom_simulation.get_implied_price(), 25000000.0 / (total_supply * 0.30));
    }
}