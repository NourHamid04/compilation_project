; ModuleID = 'program.ll'
source_filename = "program.ll"

define i64 @func(i64 %in) {
entry:
  br label %block0

block0:                                           ; preds = %entry
  br label %block1

block1:                                           ; preds = %block0
  %tmp1 = mul i64 0, 30
  %tmp2 = sub i64 9, %tmp1
  br label %block2

block2:                                           ; preds = %block1
  br label %block3

block3:                                           ; preds = %block2
  %tmp5 = add i64 5, %tmp2
  ret i64 %tmp5
}
