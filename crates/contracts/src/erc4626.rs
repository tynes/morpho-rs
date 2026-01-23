//! ERC4626 interface definitions.

use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IERC4626 {
        function deposit(uint256 assets, address receiver) external returns (uint256 shares);
        function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);
        function asset() external view returns (address);
        function totalAssets() external view returns (uint256);
        function convertToShares(uint256 assets) external view returns (uint256 shares);
        function convertToAssets(uint256 shares) external view returns (uint256 assets);
        function maxDeposit(address receiver) external view returns (uint256 maxAssets);
        function maxWithdraw(address owner) external view returns (uint256 maxAssets);
    }
}
