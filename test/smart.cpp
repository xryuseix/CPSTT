#include <iostream>
using namespace std;
int main() {
    cout << "Hello from CPP!" << endl;
    int a, b;
    cin >> a >> b;
    cout << "OUTPUT: " << a << " " << b << endl;
    if (a && b) {
        cout << string(50, 'A') << endl;
    }
}