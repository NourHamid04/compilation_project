define i64 @func(i64 %in) {
entry:
  %a_ptr = alloca i64
  %b_ptr = alloca i64
  %in_ptr = alloca i64
  %out_ptr = alloca i64
  %x_ptr = alloca i64
  store i64 0, ptr %a_ptr
  store i64 0, ptr %b_ptr
  store i64 %in, ptr %in_ptr
  store i64 0, ptr %out_ptr
  store i64 0, ptr %x_ptr
  br label %block0
block0:
  store i64 30, ptr %a_ptr
  br label %block1
block1:
  %tmp0 = load i64, ptr %a_ptr
  %tmp1 = mul i64 0, %tmp0
  %tmp2 = sub i64 9, %tmp1
  store i64 %tmp2, ptr %b_ptr
  br label %block2
block2:
  store i64 5, ptr %x_ptr
  br label %block3
block3:
  %tmp3 = load i64, ptr %x_ptr
  %tmp4 = load i64, ptr %b_ptr
  %tmp5 = add i64 %tmp3, %tmp4
  store i64 %tmp5, ptr %out_ptr
  %tmp6 = load i64, ptr %out_ptr
  ret i64 %tmp6
}
