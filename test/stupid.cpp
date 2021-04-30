#include <iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << "OUTPUT: " << a << " " << b << endl;
    if (a && b) {
        cout << string(50, 'A') << endl;
        // int ans = 0;
        // for (int i = 0; i < 100000; i++) {
        //     for (int j = 0; j < 100000; j++) {
        //         ans += (i * 12 + j) % 11;
        //         ans %= 11;
        //     }
        // }
        // cout << ans << endl;
    }
}