# レインボーテーブル攻撃

### 概要
レインボーテーブル攻撃の簡易的な実装です。
利用した平文は以下です。
https://github.com/danielmiessler/SecLists/blob/master/Passwords/Common-Credentials/10-million-password-list-top-1000000.txt

各チェーンの長さは300としています。なので衝突を無視すると
100000 * 300 = 30000000
パターンのパスワードを復元できるはず...
