# G3 単一block局所編集pilot結果

- 実施日：2026-11-13
- 状態：**限定pilotを実施。AIC-J/0.1は未採用候補のまま。**
- 事前条件：[G3 protocol](../proposals/G3_単一block局所編集比較プロトコル案.md)、[実行artifact](../../experiments/g3-edit-pilot-2026-11-13/)
- 意味範囲：E1–E6のu64単一block局所編集だけ。branch/Phi/loopや他の言語・課題へ一般化しない。

## 条件と再現情報

採用済みAIC/0.2を対照、pilot専用profile AIC-J/0.1を未採用候補として、E1–E6を各10回、計60 paired slotsで実行した。modelは`Qwen/Qwen3-4B-GGUF` revision `bc640142c66e1fdd12af0bd68f40445458f3869b`、`Qwen3-4B-Q4_K_M.gguf`（2,497,280,256 bytes、SHA-256 `7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5`）、Apache-2.0。`llama-cpp-python==0.3.16`でCPU 5 threads、4096 context、最大1024 output tokens、temperature 0.7、top-p 0.8、top-k 20、min-p 0、presence penalty 1.5、pair内共通seed、hash由来の実行順を使用した。tool/web/remote inferenceは使っていない。

事前登録manifest SHA-256は`af2cf6307753ba44a29f664b3958552146ec9058ae10263c9e1747796bdabdfd`。model chat-template SHA-256は`57f1fd00f0013a2be96aa79b857391f27e23df5b5f847072b524c897e24d0361`。prompt packet 12件、scorer/adapter、production parser/validator等のsource hashはmanifestに格納した。

外部一次資料は[pinしたHugging Face model card](https://huggingface.co/Qwen/Qwen3-4B-GGUF/blob/bc640142c66e1fdd12af0bd68f40445458f3869b/README.md)と[同revisionのGGUF artifact](https://huggingface.co/Qwen/Qwen3-4B-GGUF/blob/bc640142c66e1fdd12af0bd68f40445458f3869b/Qwen3-4B-Q4_K_M.gguf)。license表示、revision、download filenameと実bytes hashを分けて記録した。推論はlocal-only。

## 観測結果

| format | slot | 完了response | interrupted unknown | strict parse受理 | validator受理 | task edit一致 | model回答object/consumer検査 |
|---|---:|---:|---:|---:|---:|---:|---:|
| AIC/0.2 | 60 | 60 | 0 | 0 | 0 | 0 | 0 |
| AIC-J/0.1 | 60 | 59 | 1 | 0 | 0 | 0 | 0 |

全119回答は、回答本文先頭に空の`<think>\n\n</think>\n\n` wrapperを含んでいた。AIC回答はその後に`AIC 0.2`、JSON回答はその後に`{`で始まるobjectを返したが、固定した「完全なprogramを一つだけ」というstrict入力条件ではwrapperを含む回答は形式不正となる。このためAIC production parserとJSON strict parserのどちらも受理0件となった。固定scorerは内容を除去・修復せず、そのまま判定した。

全taskで主指標のpilot successは0/10 slotsだった。E2 replicate 2のJSON call index 24は、最初のprocessの外部中断時に応答が保存されていなかった。再送せず`interrupted_unknown`として1 slotに記録し、同pairはJSON回答なしとして集計した。初回24回答の内容・得点を見ずに回復手順を固定し、call 25–119を続行した。全120 slotは連番で保存され、119 response完了・1 unknown、inference exceptionは0件。累積の最後の記録時点は3,960.61秒（約66分）で、4時間上限内だった。

完了回答の平均token数・時間は、AIC/0.2が入力485.8、出力67.7、平均21.4秒、JSONが入力663.0、出力148.2、平均39.0秒（59応答）。両者は同一tokenizerなので記述値として比較できるが、format成功が0件のため編集効率の実質比較とは扱わない。最大completionは154 tokensだった。

形式不正で全モデル回答がproduction validatorへ到達しなかったため、候補回答からのobject compile、C++ ABI consumer、22入力oracle比較は実施されていない。合成E1 fixtureによる事前preflightでは両formatともparser/validator、production compiler、既存ABI harness、独立Python oracleの全22入力を通過した（[preflight record](../../experiments/g3-edit-pilot-2026-11-13/preflight.json)）。これはモデル回答の実物検査に代わるものではない。

## 判定範囲

この結果は、固定したQwen3 GGUF chat-template、`/no_think` system message、prompt packet、raw output strictnessの組合せが、この限定pilotではformat gateを通らなかったことを示す。AIC/0.2とJSON候補の意味保存やcompiler品質の同等性・優劣を示さない。paired success差が各taskで0なのも、両形式のstrict parseが0であることの帰結であり、形式間の同等性やwinnerを示さない。

chat-template設定を変える、空think wrapperを除く、回答を修復する、または別modelを使う追試は今回の結果へ混ぜない。必要なら変更した条件を別pilotとして事前登録する。AIC-J/0.1の正式採用、QA閾値、G3全体の完了、β公開判断は行っていない。

## Artifact map

- `experiments/g3-edit-pilot-2026-11-13/trials.jsonl`: raw prompt/response, token usage, seed/order, timestamps, unknown slot。
- `experiments/g3-edit-pilot-2026-11-13/scored.jsonl`, `summary.json`: scorer stage結果、task別・形式別記述集計。
- `experiments/g3-edit-pilot-2026-11-13/raw_audit.json`: 120-slot、schedule、prompt/response-byte監査。
- `experiments/g3-edit-pilot-2026-11-13/recovery_addendum.json`, `recovery_status.json`: interruption recoveryと累積budgetの記録。
- model weights、venv、Cargo builds、合成preflight object/consumerは`/tmp/aic-g3-pilot-2026-11-13/`に保管し、repositoryには含めない。
- source archive、取得・生成物、外部保管物、trashの5-state棚卸しと件数・byte合計は[資産inventory](G3_資産棚卸_2026-11-13.json)。
