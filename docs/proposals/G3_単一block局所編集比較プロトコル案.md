# G3 単一block局所編集の比較プロトコル案

- 作成日：2026-11-13
- 状態：**未採用候補に限る限定pilot。Ownerは目的に基づく条件選択・実行を委任した。2026-11-13時点で条件を選定、試行前。G3全体、製品仕様、QA閾値、正式採用を確定しない。**
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

全trialの共通promptは次の固定文とする。形式別schema card、課題文、初期programの順で渡し、同じ課題/replicate内で形式以外の入力を変えない。

```text
提示されたprogramに指定taskを適用してください。要求された変更を実施し、要求に含まれない意味、操作、識別子は保ってください。
回答は指定形式の完全なprogramを1つだけ返し、説明文やMarkdown fenceを付けないでください。
```

pilotのuser-message bytesは、UTF-8/no BOMで`COMMON_PROMPT + "\n\n" + SCHEMA_CARD + "\n\n" + TASK_REQUEST + "\n\n" + INITIAL_PROGRAM`と連結し、末尾newlineを加えない。SCHEMA_CARD/TASK_REQUEST/INITIAL_PROGRAMの内部bytesは以下の固定block/tableから取得する。system messageは両形式で`/no_think`の1件、developer messageなしとし、同一のQwen3 chat templateで送る。

形式別schema card本文は次の固定文だけを使い、試行時に説明を足したり削ったりしない。両cardは関数意味、operation subset、ID規則、完全program出力条件を伝える。comment fidelityはこのpilotでは測らないため、両形式の入力と出力にcommentを含めない。

**AIC/0.2 card:**

```text
AIC/0.2の完全なmoduleを一つ返してください。出力形は次のとおりです。
AIC 0.2
fn aic_main(%x: u64) -> u64 {
^entry:
  %id = const.u64 DECIMAL
  %id = add.wrap.u64 %lhs, %rhs
  %id = sub.wrap.u64 %lhs, %rhs
  ret.u64 %id
}
const.u64行、add.wrap.u64行、sub.wrap.u64行は必要な数だけ使い、各instruction/terminatorは1物理行に一つ置いてください。add.wrap.u64とsub.wrap.u64はmodulo 2^64です。value IDは[a-z][a-z0-9_]{0,62}、parameterとinstruction definitionは互いに異なる一意なID、参照では%を付け、definitionより前のuseを作らないでください。id/lhs/rhs/DECIMALは説明用placeholderなので実際のID/数値へ置き換え、placeholderそのものは出力しないでください。DECIMALは`0`、または1〜20桁で1〜9から始まり、値が18446744073709551615以下の符号なし10進数です。追加のfunction、block、operation、commentは出力しないでください。
```

**AIC-J/0.1 card:**

```text
AIC-J/0.1の完全なJSON objectを一つ返してください。top-levelはformat="AIC-J/0.1"とfunctionだけです。function keyはname、params、result、blocksだけで、name="aic_main"、paramsは[{"id":"x","type":"u64"}]、result="u64"、blocksはid="entry"の一要素です。block keyはid、ops、termだけです。opsは以下のshapeを使います：const `{ "id": ID, "op": "const.u64", "value": DECIMAL_STRING }`、add `{ "id": ID, "op": "add.wrap.u64", "args": [ID, ID] }`、sub `{ "id": ID, "op": "sub.wrap.u64", "args": [ID, ID] }`。add.wrap.u64とsub.wrap.u64はmodulo 2^64です。termは`{ "op": "ret.u64", "value": ID }`だけです。IDは[a-z][a-z0-9_]{0,62}、parameterとinstruction definitionは互いに異なる一意なID、JSONでは%や^を付けません。IDとDECIMAL_STRINGは説明用placeholderなので実際の値に置き換え、placeholderそのものは出力しないでください。DECIMAL_STRINGは`0`、または1〜20桁で1〜9から始まり、値が18446744073709551615以下の符号なし10進数です。array順を保ち、definitionより前のuseを作らないでください。duplicate key、unknown key/operation、型やarityの誤り、重複ID、未解決参照、範囲外定数は不正です。
```

初期programはAIC/0.2では上記のsource、AIC-J/0.1では次のbytesとする。object key順とindentationは入力hash/token数を固定するためのserial formであり、候補profile上の意味順ではない。

```json
{
  "format": "AIC-J/0.1",
  "function": {
    "name": "aic_main",
    "params": [{"id": "x", "type": "u64"}],
    "result": "u64",
    "blocks": [
      {
        "id": "entry",
        "ops": [
          {"id": "one", "op": "const.u64", "value": "1"},
          {"id": "sum", "op": "add.wrap.u64", "args": ["x", "one"]}
        ],
        "term": {"op": "ret.u64", "value": "sum"}
      }
    ]
  }
}
```

task requestと期待するordered operation listを以下に固定する。`const(id,value)`、`add(id,lhs,rhs)`、`sub(id,lhs,rhs)`、`ret(id)`は正規化表記で、AICでは`%`を含む参照へ、JSONでは文字列IDへ対応する。task target欄にないnode/fieldの変更は依頼外変更として数える。operation listの順序は意味を持つ。

| ID | 固定するtask request | task target | 期待するordered operation list |
|---|---|---|---|
| E1 | `identifier oneで定義されている定数を1から17に変更してください。ほかの操作は変更しないでください。` | `const(one).value` | `const(one,17); add(sum,x,one); ret(sum)` |
| E2 | `sumの演算をwrap減算に変えてください。operandと定数は変更しないでください。` | `sum.kind` | `const(one,1); sub(sum,x,one); ret(sum)` |
| E3 | `現在のsumの値から2を引いた値を返してください。定数をidentifier two、結果をidentifier resultで追加してください。` | `const(two)`, `sub(result,sum,two)`, return reference | `const(one,1); add(sum,x,one); const(two,2); sub(result,sum,two); ret(result)` |
| E4 | `sumという値identifierをanswerへ変更し、その定義とすべての参照を更新してください。演算と定数は変えないでください。` | `sum` definition/reference IDs | `const(one,1); add(answer,x,one); ret(answer)` |
| E5 | `入力値に関係なく7を返すprogramに変更し、計算に不要になった操作を削除してください。定数identifierはsevenにしてください。` | all operations and return | `const(seven,7); ret(seven)` |
| E6 | `定数oneを4に変え、現在のsumに2を足した値を返してください。追加する定数と結果のidentifierはtwoとresultにしてください。` | `const(one).value`, new `const(two)`/`add(result)` and return reference | `const(one,4); add(sum,x,one); const(two,2); add(result,sum,two); ret(result)` |

形式別schema card、各taskのUTF-8 bytes、format別初期program bytes、task bank順、parser/validator/oracle versionとSHA-256を最初のmodel call前に保存し、試行中は変更しない。system/developer message、model/provider/revision、sampling設定、context/output上限、tool無効条件を全callで固定する。

この6カードは実験手順を試す小標本であり、実用課題bankの代表性や網羅性を示さない。期待operation listや固定promptはmodelへ渡さず、scorerにのみ保持する。

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

### 事前選定・Owner委任の記録

2026-11-13にOwnerは、pilotを進めるか、候補profile/task bank、model、実行予算、raw artifact保持条件を目的に照らして実行者が選び実行するよう指示した。以下はその権限範囲で固定する選定であり、AIC-J/0.1の製品採用、G3全体の完了、QA閾値、β公開判断ではない。

- **比較対象：** 採用済み対照AIC/0.2対未採用候補AIC-J/0.1。このpilot専用JSON profileは§2のschema card本文に固定する。課題はE1–E6の6件だけを使う。
- **規模：** 各task 10 paired run、計60 pair/120 first-response calls。task IDはASCII `E1`–`E6`、replicateは`0..9`。pair seedは`SHA-256(ASCII("AIC-G3-pilot-seed-v1") || 0x00 || task_id || 0x00 || uint32_be(replicate))`の先頭4 byteをunsigned big-endian整数として使う。同じseedをpair内の両formatへ使う。順序bitは同様にprefix `AIC-G3-pilot-order-v1`で生成したhashの先頭byteの最下位bitとし、0ならAICを先、1ならJSONを先に実行する。修復turnは行わず、各callのmax outputは1024 token。推論失敗はretryせず失敗記録へ含める。
- **model/provider：** Hugging Faceの`Qwen/Qwen3-4B-GGUF` revision `bc640142c66e1fdd12af0bd68f40445458f3869b`、`Qwen3-4B-Q4_K_M.gguf`をlocal CPUで実行する。Model card licenseはApache-2.0。samplingはmodel cardのnon-thinking推奨（temperature 0.7、top_p 0.8、top_k 20、min_p 0、presence_penalty 1.5）、context 4096、system message `/no_think`。Tool/web accessはなく、remote inference requestやtoken課金は使わない。Runnerは`llama-cpp-python==0.3.16`に固定する。
- **実行予算：** 最大120 generation call、各1回のみ。CPUは現environmentの5 threadを使用し、実行壁時計4時間で終了する。model fileは2,497,280,256 bytesを上限にdownloadし、推論費用は0。上限を超えた場合は部分結果を明示して止める。
- **判定・object：** 指定のstrict parser/common validator/Python modular oracleを実装して使う。parser/validator/oracleはmodel promptの外に置き、期待operation listをmodelへ渡さない。各candidate outputのraw bytes、compile/consumer結果、hashを記録する。strict parseとvalidatorを通った出力は、実compilerと同じtarget profileでobject化し、汎用C++ consumerから全22固定入力を実行する。これは隔離実験であり製品pathへ配線しない。
- **保存：** 全raw prompt/response、metadata、判定結果とrunnerを`experiments/g3-edit-pilot-2026-11-13/`へ保存し、pilot reportとともにrepositoryへ記録する。model weightsとisolated Python environment/object artifactsはrepository外の`/tmp/aic-g3-pilot-2026-11-13/`に保持し、hash/pathをreportへ記載する。model weightsはGitHubへuploadしない。
- **停止境界：** model artifactのrevision/license/hashが合わない場合、outputが固定promptと結びつかない場合、runner/adapterの生産コード流用やproduction path変更が必要になった場合は試行前に止める。候補優劣、総合勝者、製品採用、正式QA、公開の閾値は本pilotから決めない。

条件固定時点では試行結果は存在せず、AIC-J/0.1は未採用候補のままとする。
