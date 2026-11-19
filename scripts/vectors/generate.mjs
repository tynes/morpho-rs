import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { MarketUtils, MathLib, ORACLE_PRICE_SCALE, SharesMath } from "@morpho-org/blue-sdk";

const CASES = 10_000;
const OUT = join(dirname(fileURLToPath(import.meta.url)), "../../crates/sim/tests/vectors");

const U128_MAX = (1n << 128n) - 1n;
const MASK64 = (1n << 64n) - 1n;
const LLTVS = [385n, 625n, 770n, 860n, 915n, 945n, 965n, 980n].map((permille) => permille * 10n ** 15n);

let seed = 0x7469_6465_7761_7463n;
const next64 = () => {
  seed = (seed + 0x9e37_79b9_7f4a_7c15n) & MASK64;
  let z = seed;
  z = ((z ^ (z >> 30n)) * 0xbf58_476d_1ce4_e5b9n) & MASK64;
  z = ((z ^ (z >> 27n)) * 0x94d0_49bb_1331_11ebn) & MASK64;
  return z ^ (z >> 31n);
};
const randomBits = (n) =>
  Array.from({ length: Math.ceil(Number(n) / 64) }).reduce((acc) => (acc << 64n) | next64(), 0n) &
  ((1n << n) - 1n);
const bitLength = (x) => BigInt(x.toString(2).length);
const below = (n) => (n <= 0n ? 0n : randomBits(bitLength(n) + 64n) % n);
const chance = (p) => Number(next64() >> 11n) / 2 ** 53 < p;
const pick = (xs) => xs[Number(below(BigInt(xs.length)))];
const logUniform = (maxBits) => {
  const bits = below(BigInt(maxBits) + 1n);
  return bits === 0n ? 0n : randomBits(bits) | (1n << (bits - 1n));
};
const clamp128 = (x) => (x > U128_MAX ? U128_MAX : x);
const positive = (x) => (x === 0n ? 1n : x);
const upTo = (max) => {
  const choice = below(4n);
  if (choice === 0n) return 0n;
  if (choice === 1n) return max;
  if (choice === 2n) return below(max + 1n);
  const x = logUniform(bitLength(max));
  return x > max ? below(max + 1n) : x;
};

const randomLltv = () => (chance(0.7) ? pick(LLTVS) : 385n * 10n ** 15n + below(595n * 10n ** 15n + 1n));

const randomPrice = () => {
  if (chance(0.01)) return undefined;
  if (chance(0.02)) return 0n;
  if (chance(0.05)) return ORACLE_PRICE_SCALE;
  return logUniform(160);
};

const randomMarket = () => {
  const totalBorrowAssets = chance(0.04) ? 0n : chance(0.04) ? U128_MAX : logUniform(128);
  const totalBorrowShares = (() => {
    if (chance(0.03)) return 0n;
    if (chance(0.03)) return U128_MAX;
    if (totalBorrowAssets > 0n && chance(0.75))
      return clamp128((totalBorrowAssets * 10n ** 6n * (1000n - below(500n))) / 1000n);
    return logUniform(128);
  })();
  return {
    totalSupplyAssets: clamp128(totalBorrowAssets + (chance(0.1) ? 0n : logUniform(128))),
    totalBorrowAssets,
    totalBorrowShares,
    price: randomPrice(),
    lltv: randomLltv(),
  };
};

const liquidationThreshold = (borrowShares, market) =>
  MathLib.mulDivUp(
    MathLib.wDivUp(MarketUtils.toBorrowAssets(borrowShares, market, "Up"), market.lltv),
    ORACLE_PRICE_SCALE,
    market.price,
  );

const randomPosition = (market, healthyBias) => {
  const borrowShares = chance(0.04)
    ? 0n
    : healthyBias
      ? upTo(market.totalBorrowShares)
      : 1n + below(market.totalBorrowShares);
  const collateral = (() => {
    if (market.price > 0n && borrowShares > 0n && chance(healthyBias ? 0.75 : 0.95)) {
      const threshold = liquidationThreshold(borrowShares, market);
      if (chance(0.1)) return clamp128(threshold - below(2n));
      const permille = healthyBias ? 500n + below(1001n) : 300n + below(701n);
      return clamp128((threshold * permille) / 1000n);
    }
    return pick([0n, 1n, U128_MAX, logUniform(128), logUniform(128)]);
  })();
  return { collateral, borrowShares };
};

const json = (value) => (value == null ? null : typeof value === "boolean" ? value : value.toString());

const marketJson = (market) => ({
  total_supply_assets: json(market.totalSupplyAssets),
  total_borrow_assets: json(market.totalBorrowAssets),
  total_borrow_shares: json(market.totalBorrowShares),
  price: json(market.price),
  lltv: json(market.lltv),
});

const positionJson = (position) => ({
  collateral: json(position.collateral),
  borrow_shares: json(position.borrowShares),
});

const rawMathCase = () => {
  const market = randomMarket();
  const position = randomPosition(market, true);
  const params = { lltv: market.lltv };
  const seizable = MarketUtils.getSeizableCollateral(position, market, params);
  const repaidSharesIn = chance(0.8) ? upTo(position.borrowShares) : upTo(market.totalBorrowShares);
  const seizedAssetsIn = pick([0n, position.collateral, seizable ?? 0n, upTo(position.collateral)]);
  return {
    market: marketJson(market),
    position: positionJson(position),
    repaid_shares_in: json(repaidSharesIn),
    seized_assets_in: json(seizedAssetsIn),
    liquidation_incentive_factor: json(MarketUtils.getLiquidationIncentiveFactor(params)),
    collateral_value: json(MarketUtils.getCollateralValue(position.collateral, market)),
    max_borrow_assets: json(MarketUtils.getMaxBorrowAssets(position.collateral, market, params)),
    is_healthy: json(MarketUtils.isHealthy(position, market, params)),
    health_factor: json(MarketUtils.getHealthFactor(position, market, params)),
    liquidation_price: json(MarketUtils.getLiquidationPrice(position, market, params)),
    liquidation_seized_assets: json(MarketUtils.getLiquidationSeizedAssets(repaidSharesIn, market, params)),
    liquidation_repaid_shares: json(MarketUtils.getLiquidationRepaidShares(seizedAssetsIn, market, params)),
    liquidation_repaid_assets: json(MarketUtils.toBorrowAssets(repaidSharesIn, market, "Up")),
    seizable_collateral: json(seizable),
  };
};

class Revert extends Error {}
const revert = (reason) => {
  throw new Revert(reason);
};
const sub = (x, y) => (x < y ? revert("LiquidationExceedsPosition") : x - y);

// Port of `Morpho.liquidate` after interest accrual, on blue-sdk math.
const liquidate = (market, position, seizedIn, repaidIn) => {
  if ((seizedIn === 0n) === (repaidIn === 0n)) revert("InconsistentLiquidationInput");
  if (market.price == null) revert("UnknownOraclePrice");
  const params = { lltv: market.lltv };
  if (position.borrowShares === 0n || MarketUtils.isHealthy(position, market, params)) revert("HealthyPosition");
  const lif = MarketUtils.getLiquidationIncentiveFactor(params);
  const { totalBorrowAssets, totalBorrowShares } = market;

  let seizedAssets = seizedIn;
  let repaidShares = repaidIn;
  if (seizedAssets > 0n) {
    const seizedQuoted = MathLib.mulDivUp(seizedAssets, market.price, ORACLE_PRICE_SCALE);
    repaidShares = SharesMath.toShares(MathLib.wDivUp(seizedQuoted, lif), totalBorrowAssets, totalBorrowShares, "Up");
  } else {
    if (market.price === 0n) revert("DivisionByZero");
    const repaid = SharesMath.toAssets(repaidShares, totalBorrowAssets, totalBorrowShares, "Down");
    seizedAssets = MathLib.mulDivDown(MathLib.wMulDown(repaid, lif), ORACLE_PRICE_SCALE, market.price);
  }
  const repaidAssets = SharesMath.toAssets(repaidShares, totalBorrowAssets, totalBorrowShares, "Up");

  const after = {
    totalSupplyAssets: market.totalSupplyAssets,
    totalBorrowShares: sub(totalBorrowShares, repaidShares),
    totalBorrowAssets: MathLib.zeroFloorSub(totalBorrowAssets, repaidAssets),
  };
  const remaining = {
    borrowShares: sub(position.borrowShares, repaidShares),
    collateral: sub(position.collateral, seizedAssets),
  };

  let badDebtAssets = 0n;
  let badDebtShares = 0n;
  if (remaining.collateral === 0n) {
    badDebtShares = remaining.borrowShares;
    badDebtAssets = MathLib.min(
      after.totalBorrowAssets,
      SharesMath.toAssets(badDebtShares, after.totalBorrowAssets, after.totalBorrowShares, "Up"),
    );
    after.totalBorrowAssets -= badDebtAssets;
    after.totalSupplyAssets = sub(after.totalSupplyAssets, badDebtAssets);
    after.totalBorrowShares = sub(after.totalBorrowShares, badDebtShares);
    remaining.borrowShares = 0n;
  }

  return {
    market: {
      total_supply_assets: json(after.totalSupplyAssets),
      total_borrow_assets: json(after.totalBorrowAssets),
      total_borrow_shares: json(after.totalBorrowShares),
    },
    position: positionJson(remaining),
    result: {
      repaid_assets: json(repaidAssets),
      repaid_shares: json(repaidShares),
      seized_assets: json(seizedAssets),
      bad_debt_assets: json(badDebtAssets),
      bad_debt_shares: json(badDebtShares),
    },
  };
};

const rawLiquidateCase = () => {
  const market = randomMarket();
  const position = randomPosition(market, chance(0.05));
  const seizable = MarketUtils.getSeizableCollateral(position, market, { lltv: market.lltv }) ?? 0n;
    const [seizedIn, repaidIn] = (() => {
    if (chance(0.03)) return [0n, 0n];
    if (chance(0.03)) return [positive(upTo(position.collateral)), positive(upTo(position.borrowShares))];
    if (chance(0.5)) {
      const seized = chance(0.05)
        ? position.collateral + 1n
        : pick([seizable, seizable, seizable, position.collateral, 1n + below(seizable), 1n + below(seizable)]);
      return [positive(seized), 0n];
    }
    const repaid = chance(0.05)
      ? position.borrowShares + 1n
      : pick([position.borrowShares, 1n + below(position.borrowShares), 1n + below(position.borrowShares)]);
    return [0n, positive(repaid)];
  })();
  const expected = (() => {
    try {
      return liquidate(market, position, seizedIn, repaidIn);
    } catch (error) {
      if (error instanceof Revert) return { error: error.message };
      throw error;
    }
  })();
  return {
    market: marketJson(market),
    position: positionJson(position),
    seized_assets_in: json(seizedIn),
    repaid_shares_in: json(repaidIn),
    expected,
  };
};

const U256_MAX = (1n << 256n) - 1n;
const fitsUint256 = (value) =>
  typeof value === "string"
    ? !/^\d+$/.test(value) || BigInt(value) <= U256_MAX
    : value == null || typeof value !== "object" || Object.values(value).every(fitsUint256);
// Morpho reverts on uint256 overflow; such states cannot occur on chain.
const valid = (generate) => () => {
  const candidate = generate();
  return fitsUint256(candidate) ? candidate : valid(generate)();
};
const mathCase = valid(rawMathCase);
const liquidateCase = valid(rawLiquidateCase);

const write = (name, cases) => {
  const body = `{"source":"@morpho-org/blue-sdk 8.3.0","cases":[\n${cases.map((c) => JSON.stringify(c)).join(",\n")}\n]}\n`;
  writeFileSync(join(OUT, name), body);
  console.log(`${name}: ${cases.length} cases`);
};

mkdirSync(OUT, { recursive: true });
write("market_math.json", Array.from({ length: CASES }, mathCase));
write("liquidate.json", Array.from({ length: CASES }, liquidateCase));
