# G3 単一block局所編集の比較プロトコル案

- 作成日：2026-11-13
- 状態：**未採用の限定プロトコル案。実験未実施。G3全体、候補選定、製品仕様、QA閾値を確定しない。**
- 対照：Owner採用済みAIC/0.2。
- 比較候補：未採用AIC-J/0.1。
- 関連資料：[初期変換契約](01_初期変換契約案.md)、[G1単一課題観測](../evidence/G1_単一課題観測_2026-11-13.md)、[構造化編集と版進化の限定比較](../research/04_構造化編集・版進化比較.md)。

## 1. 問いと範囲

同じ純粋なu64関数の局所編集をAIへ依頼したとき、AIC/0.2の行指向textとAIC-J/0.1のJSON候補で、初回に要求どおりの受理可能な意味を返す割合と編集コストに差が現れるかを探索する。対象は**単一block内の定数・wrap加減算・returnだけ**とする。AIC/0.2全体の優位性、branch/Phi/loopを含む形式一般の比較、既存compilerに対する性能比較は問わない。

この切り出しは実装済みの意味を共通土台として使うための実験境界である。branch、Phi、loop、比較演算、他target、memory、call、I/Oは候補から排除せず、このpilotの対象外とする。実験結果が得られても他の機能・課題へ一般化しない。

## 2. 比較する入力条件

両候補とも`aic_main(u64) -> u64`の純粋関数とし、u64定数、`add.wrap.u64`、`sub.wrap.u64`、returnのみを許す。加減算は`2^64`を法とする。初期moduleは候補ごとに同じ意味木から作成する。タスク文、意味要件、識別子、命令の依存関係は揃え、表現別のschemaカードだけを変える。

### AIC/0.2対照

現行の採用済み文法・validatorをそのまま使う。pilot入力は`aic_main`、引数`%x: u64`、block `^entry`一つ、AIC/0.2で許される定数・wrap加減算・returnに制限する。既存契約の変更や互換性拡張は行わない。

### AIC-J/0.1候補

以下はこのpilot用の**未採用候補profile**であり、正式schemaや製品契約ではない。

- RFC 8259 JSON objectを使い、top-levelは`format`と`function`だけ。`format`は文字列`AIC-J/0.1`。
- functionは`name: "aic_main"`、`params: [{"id":"x","type":"u64"}]`、`result: "u64"`、`blocks`を持つ。`blocks`は`id: "entry"`の一要素だけとする。
- blockは命令配列`ops`とterminator `term`を持つ。定数命令は`{"id":ID,"op":"const.u64","value":DECIMAL_STRING}`、加減算命令は`{"id":ID,"op":"add.wrap.u64"|"sub.wrap.u64","args":[ID,ID]}`、returnは`{"op":"ret.u64","value":ID}`とする。
- IDはAIC/0.2と同じ`[a-z][a-z0-9_]{0,62}`。JSON内では`%`や`^`を付けない。定数は符号なし10進文字列で、u64範囲内とする。
- objectの順序は意味に使わず、arrayの順序は保持する。duplicate key、unknown key/operation、型・arity不一致、重複ID、未解決参照、範囲外定数、複数blockは拒否する。命令の定義・参照条件はAIC/0.2の対応する条件と一致させる。

比較実装はJSONを正規化した共通operation listへ読み込む。AIC/0.2側も同じoperation listへ読み込む。共通validator・意味oracleは表現名を見ずに同じ判定を行う。AIC-J parser/adapterと独立判定器は現行repositoryにないため、実験前に隔離G4作業として実装・レビューが必要であり、この文書の追加だけでは実験を開始しない。

## 3. 事前固定するtask bank

初期programは次の意味を持つ。AIC/0.2とJSONで対応するsourceを作り、要求文は同一にする。

```text
AIC 0.2
fn aic_main(%x: u64) -> u64 {
^entry:
  %one = const.u64 1
  %sum = add.wrap.u64 %x, %one
  ret.u64 %sum
}
```

| ID | 要求する局所編集 | 正規化後に満たす条件 |
|---|---|---|
| E1 | 加算定数を1から17へ変更する | `x + 17`を返す |
| E2 | 加算を減算へ変更し、定数1は保つ | `x - 1`を返す |
| E3 | 現在の結果からさらに2を減算する | `(x + 1) - 2`を返す |
| E4 | `%sum`を`%answer`へ改名する | definitionと全referenceを更新し、意味は`x + 1`のまま |
| E5 | 常に7を返すよう変更する | 不要になった命令を除き、定数7を返す |
| E6 | 1を4へ変更し、既存結果に2を加える | `(x + 4) + 2`を返す |

各カードは試行前にformat別の初期source、同一要求文、期待する正規化条件、対象node集合を固定する。要求と期待条件は結果を見てから変更しない。この6カードは実験手順を試す小標本であり、実用課題bankの代表性や網羅性を示さない。

## 4. 判定指標とoracle

**主指標**は、初回回答が (a) 対象formatの厳密parserに受理され、(b) 共通validatorを通り、(c) 指定編集条件を満たし、(d) 全固定入力で独立意味oracleと一致したtrialの割合とする。各stageの失敗数も別々に記録し、合成スコアだけに畳み込まない。

oracleは正規化operation listを用い、wrap加減算をPythonの整数演算から`value % 2**64`で計算する別実装とする。全カードで同じ固定入力集合`{0, 1, 2, 17, 2^64-2, 2^64-1}`と、次の決定的規則で生成した16個のu64値を使う：`i = 0..15`について`SHA-256(ASCII("AIC-G3-edit-pilot-v1") || 0x00 || uint32_be(i))`の先頭8 byteをunsigned big-endian整数として使う。生成入力と期待出力はtask bankへ実験前に保存する。実行objectの挙動も確認する場合は、同一compiler/build、target、consumer、flagsを固定し、source-level oracleとの一致を別欄で報告する。既存interpreterとcompiler出力だけの相互一致を独立証拠とは数えない。

副指標は、初回parse受理率、意味一致率、依頼外の正規化node変更数、model入力/出力token、応答byte数、修復turn数とする。編集量はformat固有の行数やJSON key数で比較せず、共通operation list上で対象node外に生じた変更として数える。token数は同一provider tokenizerが取得できる場合のみ直接比較する。異なるtokenizerしかない場合は別々に示し、大小を直接比較しない。

初回回答を主指標に使う。補助修復を測る場合は、同じ最大2 turn、同じerror-class情報（parse、validation、意味条件）、同じ修復指示を両候補へ適用し、修復後成功率とturn数を主指標から分離する。秘密の期待出力や他形式の正解sourceはmodelへ渡さない。

## 5. 実行条件と報告

実験案の既定値は**6 task × 10 paired run = 60 paired trials（各形式60回答、合計120初回回答）**とする。各pairで同じtask要求を使い、形式の実行順をrunごとに無作為化する。同一model ID/revision、system/developer指示、sampling設定、context/output上限を固定し、tool利用とweb閲覧は両形式とも無効にする。APIがseedを提供する場合はpair内で同じseedを使い、提供しない場合はその事実を記録する。これは探索pilotの規模であり、一般的性能差を確定するサンプル数ではない。

試行前にmodel ID/revision、provider、sampling・seed条件、prompt/schemaカード、tokenizer、task bank hash、adapter/oracle commit、実行環境を記録する。全raw prompt/response、metadata、parser/validator結果、oracle出力、修復会話、実行objectを保存し、取得不能項目は`UNKNOWN`または`NOT_AVAILABLE`とする。

報告では6 task別の成功数・分母、全体の記述率、paired差、失敗分類、token/編集量を併記する。重み付き総合点や一形式の勝者は決めない。信頼区間や統計的検定を追加する場合も、task数の少ない探索結果として明記し、独立試行・一般化を過大に主張しない。性能閾値、採用基準、QA合否はOwnerが別途決める。

## 6. G3/G4/G5/G6境界

この案はG3の「単一block局所編集」評価方法を具体化するだけで、G3全体の完了を意味しない。比較対象にAIC/0.2以外の候補を追加するか、JSON候補profileを承認するかは未決定のままとする。評価adapter、実験用task、結果は採用済みcompiler pathから隔離する。G4で隔離実装し、G5では同じ条件の実object/consumer検査を別に行い、G6でOwnerが採否・閾値・対象範囲を判断する。正式QA、製品公開、最適性の主張は本案の範囲外である。

### Ownerが実験開始前に決める事項

- 比較候補profileとtask bankをこのpilotに限って使うこと。
- 実験に使うmodel/provider、実行予算、raw artifactの保存先と保持期間。
- 10 paired run/taskの規模を受け入れるか、変更するか。
- G4隔離実装とG5 object/consumer検査を開始する条件。

これらの判断が記録されるまで、実験結果は存在せず、AIC-J/0.1は未採用候補のままとする。
