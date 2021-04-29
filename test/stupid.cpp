#include <iostream>
using namespace std;
int main() {
    int a, b;
    cin >> a >> b;
    cout << "I'm Stupid: " << a << " " << b << endl;
    if (a && !b) {
        for (int i = 0; i < 1000; i++) {
            cout << char('A' + i % 26) << endl;
        }
    }
}