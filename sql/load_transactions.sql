insert into raw.transactions
select
  parseDateTimeBestEffort(blockTimestamp) as blockTimestamp,
  blockHash,
  blockNumber,
  from,
  reinterpretAsInt256(reverse(unhex(substring(gas,3)))) as gas,
  reinterpretAsInt256(reverse(unhex(substring(gasPrice,3)))) as gasPrice,
  hash ,
  input ,
  nonce ,
  to ,
  reinterpretAsInt64(reverse(unhex(substring(transactionIndex,3)))) as transactionIndex,
  value,
  type ,
  chainId,
  v,
  r,
  s,
  reinterpretAsInt256(reverse(unhex(substring(maxFeePerGas,3)))) as maxFeePerGas,
  reinterpretAsInt256(reverse(unhex(substring(maxPriorityFeePerGas,3)))) as maxPriorityFeePerGas,
  accessList,
  yParity
from file('./transactions_*');

