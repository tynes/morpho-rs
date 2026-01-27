//! ERC4626 interface definitions.

use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IERC4626 {
        // Transaction functions
        function deposit(uint256 assets, address receiver) external returns (uint256 shares);
        function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);
        function mint(uint256 shares, address receiver) external returns (uint256 assets);
        function redeem(uint256 shares, address receiver, address owner) external returns (uint256 assets);

        // View functions
        function asset() external view returns (address);
        function totalAssets() external view returns (uint256);
        function convertToShares(uint256 assets) external view returns (uint256 shares);
        function convertToAssets(uint256 shares) external view returns (uint256 assets);

        // Max functions
        function maxDeposit(address receiver) external view returns (uint256 maxAssets);
        function maxWithdraw(address owner) external view returns (uint256 maxAssets);
        function maxMint(address receiver) external view returns (uint256 maxShares);
        function maxRedeem(address owner) external view returns (uint256 maxShares);

        // Preview functions
        function previewDeposit(uint256 assets) external view returns (uint256 shares);
        function previewMint(uint256 shares) external view returns (uint256 assets);
        function previewWithdraw(uint256 assets) external view returns (uint256 shares);
        function previewRedeem(uint256 shares) external view returns (uint256 assets);
    }
}
