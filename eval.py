
idxs = [0, 63, 17]

for i in idxs:
  i_rank = i / 8
  i_file = i % 8

  idx = (7 - i_rank) * 8 + i_file
  print(idx, idx & 56)