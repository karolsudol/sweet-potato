
insert into raw.blocks
select
	parseDateTimeBestEffort(timestamp) as timestamp ,
	number,
	reinterpretAsInt256(reverse(unhex(substring(baseFeePerGas,3)))) as baseFeePerGas,
	reinterpretAsInt256(reverse(unhex(substring(difficulty,3)))) as difficulty,
	extraData,
	reinterpretAsInt256(reverse(unhex(substring(gasLimit,3)))) as gasLimit,
	reinterpretAsInt256(reverse(unhex(substring(gasUsed,3)))) as gasUsed,
	hash,
	logsBloom,
	miner,
	mixHash,
	nonce,
	parentHash,
	receiptsRoot,
	sha3Uncles,
	reinterpretAsInt256(reverse(unhex(substring(size,3)))) as size,
	stateRoot,
	reinterpretAsInt256(reverse(unhex(substring(totalDifficulty,3)))) as totalDifficulty,
	transactionsRoot,
	uncles
from file('./blocks_*');

