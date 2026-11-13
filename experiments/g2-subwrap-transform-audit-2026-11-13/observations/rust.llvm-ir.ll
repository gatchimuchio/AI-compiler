; ModuleID = 'g2_subwrap.7b54fffbb10d944e-cgu.0'
source_filename = "g2_subwrap.7b54fffbb10d944e-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable
define dso_local noundef i64 @aic_main(i64 noundef %x) unnamed_addr #0 !guid !3 {
start:
  %_0 = add i64 %x, -1
  ret i64 %_0
}

attributes #0 = { mustprogress nofree norecurse nosync nounwind nonlazybind willreturn memory(none) uwtable "probe-stack"="inline-asm" "target-cpu"="x86-64" }

!llvm.module.flags = !{!0, !1}
!llvm.ident = !{!2}

!0 = !{i32 2, !"RtLibUseGOT", i32 1}
!1 = !{i32 7, !"uwtable", i32 2}
!2 = !{!"rustc version 1.99.1 (58cf8f9c3 2026-10-13)"}
!3 = !{i64 -7392004613222983590}
