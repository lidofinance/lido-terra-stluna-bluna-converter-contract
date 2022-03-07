import random
import matplotlib.pyplot as plt
import time
from enum import Enum


BLOCKS_PER_MINUTE = 10
BLOCKS_PER_HOUR = BLOCKS_PER_MINUTE * 60
BLOCKS_PER_DAY = BLOCKS_PER_HOUR * 24
BLOCKS_PER_WEEK = BLOCKS_PER_DAY * 7
BLOCKS_PER_MONTH = BLOCKS_PER_DAY * 30
BLOCKS_PER_YEAR = BLOCKS_PER_DAY * 365


class Operation(Enum):
    NOTHING = 0
    BOND_BLUNA = 1
    BOND_STLUNA = 2
    BOND_BOTH = 3
    CONVERT_STLUNA_TO_BLUNA = 4
    CONVERT_BLUNA_TO_STLUNA = 5


def get_change(current, previous):
    if current == previous:
        return 100.0
    try:
        return (abs(current - previous) / previous) * 100.0
    except ZeroDivisionError:
        return 0


class Block:
    def __init__(self, number, time) -> None:
        self.number = number
        self.time = time


class Hub:
    def __init__(self) -> None:
        self.total_bond_stluna = 1_500_000
        self.total_bond_bluna = 1_000_000

        self.total_issued_stluna = 1_000_000
        self.total_issued_bluna = 1_000_000

        self.recovery_fee = 0.05
        self.threshold = 1

        self.staking_apr = 0.09 / BLOCKS_PER_YEAR

        self.last_block = Block(0, 0)

    def bluna_exchange_rate(self):
        if self.total_bond_bluna == 0:
            return 1.0

        return self.total_bond_bluna / self.total_issued_bluna

    def stluna_exchange_rate(self):
        if self.total_bond_stluna == 0:
            return 1.0

        return self.total_bond_stluna / self.total_issued_stluna

    def slashing(self, amount):
        total_bonded = self.total_bond_bluna + self.total_bond_stluna
        actual_total_bonded = total_bonded - amount

        bluna_bond_ratio = self.total_bond_bluna / total_bonded
        self.total_bond_bluna = int(actual_total_bonded * bluna_bond_ratio)
        self.total_bond_stluna = actual_total_bonded - self.total_bond_bluna

        self.bluna_exchange_rate()
        self.stluna_exchange_rate()

    def bond_bluna(self, amount):
        bluna_mint_amount = int(amount / self.bluna_exchange_rate())
        bluna_mint_amount_with_fee = bluna_mint_amount

        if self.bluna_exchange_rate() < self.threshold:
            max_peg_fee = int(bluna_mint_amount * self.recovery_fee)
            required_peg_fee = (
                self.total_issued_bluna + bluna_mint_amount) - (self.total_bond_bluna + amount)
            peg_fee = min(max_peg_fee, required_peg_fee)
            bluna_mint_amount_with_fee = bluna_mint_amount - peg_fee

        self.total_bond_bluna += amount
        self.total_issued_bluna += bluna_mint_amount_with_fee
        self.bluna_exchange_rate()

        return bluna_mint_amount_with_fee

    def bond_stluna(self, amount):
        stluna_mint_amount = int(amount / self.stluna_exchange_rate())

        self.total_bond_stluna += amount
        self.total_issued_stluna += stluna_mint_amount

        self.stluna_exchange_rate()

        return stluna_mint_amount

    def update_global_index(self, block: Block):
        rewards = int(self.total_bond_stluna *
                      self.staking_apr * (block.number - self.last_block.number))
        self.total_bond_stluna += rewards

        self.last_block = block

        self.stluna_exchange_rate()
        self.bluna_exchange_rate()

    def convert_stluna_to_bluna(self, stluna_amount, simulation=False):
        threshold = self.threshold
        recovery_fee = self.recovery_fee

        denom_equiv = int(self.stluna_exchange_rate() * stluna_amount)

        bluna_to_mint = int(denom_equiv / self.bluna_exchange_rate())

        bluna_mint_amount_with_fee = bluna_to_mint
        if self.bluna_exchange_rate() < threshold:
            max_peg_fee = int(bluna_to_mint * recovery_fee)
            required_peg_fee = (self.total_issued_bluna +
                                bluna_to_mint) - (self.total_bond_bluna + denom_equiv)
            peg_fee = min(max_peg_fee, required_peg_fee)
            bluna_mint_amount_with_fee = bluna_to_mint - peg_fee

        if simulation:
            return bluna_mint_amount_with_fee

        self.total_bond_bluna += denom_equiv
        self.total_bond_stluna -= denom_equiv

        self.total_issued_bluna += bluna_mint_amount_with_fee
        self.total_issued_stluna -= stluna_amount

        self.bluna_exchange_rate()
        self.stluna_exchange_rate()

        return bluna_mint_amount_with_fee

    def convert_bluna_to_stluna(self, bluna_amount, simulation=False):
        threshold = self.threshold
        recovery_fee = self.recovery_fee

        total_bluna_supply = self.total_issued_bluna

        bluna_amount_with_fee = 0
        if self.bluna_exchange_rate() < threshold:
            max_peg_fee = int(bluna_amount * recovery_fee)
            required_peg_fee = total_bluna_supply - self.total_bond_bluna
            peg_fee = min(max_peg_fee, required_peg_fee)
            bluna_amount_with_fee = bluna_amount - peg_fee
        else:
            bluna_amount_with_fee = bluna_amount

        denom_equiv = int(self.bluna_exchange_rate() * bluna_amount_with_fee)
        stluna_to_mint = int(denom_equiv / self.stluna_exchange_rate())
        if simulation:
            return stluna_to_mint

        self.total_bond_bluna -= denom_equiv
        self.total_bond_stluna += denom_equiv
        self.total_issued_bluna -= bluna_amount_with_fee
        self.total_issued_stluna += stluna_to_mint

        self.bluna_exchange_rate()
        self.stluna_exchange_rate()

        return stluna_to_mint


class Converter:

    def __init__(self) -> None:
        self.hub = Hub()

        self.price0_cumulative_last = 0
        self.price1_cumulative_last = 0
        self.block_time_last = 0

        self.pcls0 = []
        self.pcls1 = []

        self.average_prices_0 = []
        self.average_prices_1 = []
        self.twap_last_time = 0
        self.last_pcls0 = 0
        self.last_pcls1 = 0

    def accumulate_prices(self, block: Block):
        time_elapsed = block.time - self.block_time_last

        if time_elapsed == 0:
            return None

        stluna_price = self.hub.convert_stluna_to_bluna(1_000_000, True)
        bluna_price = self.hub.convert_bluna_to_stluna(1_000_000, True)

        pcl0 = self.price0_cumulative_last + (time_elapsed * stluna_price)
        pcl1 = self.price1_cumulative_last + (time_elapsed * bluna_price)

        return (pcl0, pcl1, block.time)

    def convert_stluna_to_bluna(self, block: Block, amount):
        self.hub.convert_stluna_to_bluna(amount)

        accumulated_prices = self.accumulate_prices(block)
        if accumulated_prices:
            self.price0_cumulative_last = accumulated_prices[0]
            self.price1_cumulative_last = accumulated_prices[1]
            self.block_time_last = accumulated_prices[2]

    def convert_bluna_to_stluna(self, block: Block, amount):
        self.hub.convert_bluna_to_stluna(amount)

        accumulated_prices = self.accumulate_prices(block)
        if accumulated_prices:
            self.price0_cumulative_last = accumulated_prices[0]
            self.price1_cumulative_last = accumulated_prices[1]
            self.block_time_last = accumulated_prices[2]

    def get_cumulative_prices(self, block: Block):
        price0_cumulative_last = self.price0_cumulative_last
        price1_cumulative_last = self.price1_cumulative_last

        accumulated_prices = self.accumulate_prices(block)
        if accumulated_prices:
            price0_cumulative_last = accumulated_prices[0]
            price1_cumulative_last = accumulated_prices[1]

        return (price0_cumulative_last, price1_cumulative_last)

    def execute_block(self, block: Block, is_slashing: bool, operation: Operation, amount: int):
        if block.number % 10 == 0:
            self.hub.update_global_index(block)
        if is_slashing:
            self.hub.slashing((self.hub.total_bond_bluna +
                               self.hub.total_bond_stluna) / 1000)
        if operation == Operation.NOTHING:
            pass
        elif operation == Operation.BOND_BLUNA:
            self.hub.bond_bluna(amount)

        elif operation == operation.BOND_STLUNA:
            self.hub.bond_stluna(amount)

        elif operation == operation.BOND_BOTH:
            self.hub.bond_bluna(amount)
            self.hub.bond_stluna(amount)

        elif operation == operation.CONVERT_STLUNA_TO_BLUNA:
            if self.hub.total_issued_stluna > 10_000:
                self.convert_stluna_to_bluna(block, 5_000)

        elif operation == operation.CONVERT_BLUNA_TO_STLUNA:
            if self.hub.total_issued_bluna > 10_000:
                self.convert_bluna_to_stluna(block, 5_000)

        # plot values
        prices = self.get_cumulative_prices(block)

        if block.number % BLOCKS_PER_DAY == 0:
            price0_average = (prices[0] - self.last_pcls0) / \
                             (block.time - self.twap_last_time)
            price1_average = (prices[1] - self.last_pcls1) / \
                             (block.time - self.twap_last_time)

            self.twap_last_time = block.time

            self.last_pcls0 = prices[0]
            self.last_pcls1 = prices[1]

            self.average_prices_0.append(price0_average)
            self.average_prices_1.append(price1_average)
        else:
            self.average_prices_0.append(
                self.average_prices_0[len(self.average_prices_0) - 1])
            self.average_prices_1.append(
                self.average_prices_1[len(self.average_prices_1) - 1])

        self.pcls0.append(prices[0])
        self.pcls1.append(prices[1])


class ConverterWithBot(Converter):
    def __init__(self) -> None:
        super().__init__()

    def update_prices(self, block):
        accumulated_prices = self.accumulate_prices(block)
        if accumulated_prices:
            self.price0_cumulative_last = accumulated_prices[0]
            self.price1_cumulative_last = accumulated_prices[1]
            self.block_time_last = accumulated_prices[2]

    def execute_block(self, block: Block, is_slashing: bool, operation: Operation, amount: int):
        self.update_prices(block)
        super().execute_block(block, is_slashing, operation, amount)


class ConverterWithBlockFromHub(Converter):
    def __init__(self) -> None:
        super().__init__()

    def accumulate_prices(self, block: Block):
        time_elapsed = block.time - \
            max(self.block_time_last, self.hub.last_block.time)

        if time_elapsed == 0:
            return None

        stluna_price = self.hub.convert_stluna_to_bluna(1_000_000, True)
        bluna_price = self.hub.convert_bluna_to_stluna(1_000_000, True)

        pcl0 = self.price0_cumulative_last + (time_elapsed * stluna_price)
        pcl1 = self.price1_cumulative_last + (time_elapsed * bluna_price)

        return (pcl0, pcl1, block.time)


converter = Converter()
converter_with_bot = ConverterWithBot()
converter_with_block_from_hub = ConverterWithBlockFromHub()

random.seed(time.time())
blocks = []
stluna_exchange_rates = []

converters = (
    converter,
    converter_with_bot,
    converter_with_block_from_hub
)

for i in range(0, BLOCKS_PER_MONTH * 3):
    block = Block(i, i * 10 + random.randint(1, 5))

    is_slashing = False
    if block.number % 10 == 0:
        if random.randint(0, 100000) == 5:
            is_slashing = True

    operation = random.choice(list(Operation))
    amount = random.randint(1_000_000, 10_000_000)

    for converter in converters:
        converter.execute_block(block, is_slashing, operation, amount)

    blocks.append(block.number)
    stluna_exchange_rates.append(converters[0].hub.stluna_exchange_rate())

plt.subplot(3, 2, 1)
plt.ylabel("Exchange rate")
plt.xlabel("Block height")
plt.grid()
plt.plot(blocks, stluna_exchange_rates, label="stLuna exchange rate")
plt.legend(loc="upper left")

plt.subplot(3, 2, 2)
plt.ylabel("Prices")
plt.xlabel("Block height")
plt.grid()

plt.plot(blocks, converters[0].pcls0,
         label="Accumulated prices for stLuna (usual)")
plt.plot(blocks, converters[1].pcls0,
         label="Accumulated prices for stLuna (withbot)")

plt.legend(loc="upper left")

diffs = []
for i in range(len(converters[1].pcls0)):
    diffs.append(get_change(
        converters[1].pcls0[i], converters[0].pcls0[i]))

plt.subplot(3, 2, 3)
plt.ylabel("Usual and bot price diff")
plt.xlabel("Block height")
plt.grid()
plt.plot(blocks, diffs)

plt.subplot(3, 2, 4)
plt.ylabel("stLuna avg price")
plt.xlabel("Block height")
plt.grid()

plt.plot(blocks, converters[0].average_prices_0, label="usual")
plt.plot(blocks, converters[1].average_prices_0, label="bot")
# plt.plot(blocks, converters[2].average_prices_0, label="block from hub")

plt.legend(loc="upper left")


avg_diffs = []
for i in range(len(converters[0].average_prices_0)):
    avg_diffs.append(get_change(
        converters[0].average_prices_0[i] / 1_000_000, stluna_exchange_rates[i]))

plt.subplot(3, 2, 5)
plt.ylabel("Avg price - exchange rate diff %")
plt.xlabel("Block height")
plt.grid()
plt.plot(blocks, avg_diffs)

plt.show()
