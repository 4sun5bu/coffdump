/* このファイルは、テスト用のZ8000 COFFファイルを作るためのものです。
 * 次のように、オブジェクトファイルを作成してください。
 *   z8k-coff-gcc -mz8002 -c add.c 
 */

extern int sum;

int add(a, b)
int a, b;
{
    sum = a + b;

    return sum;
}
