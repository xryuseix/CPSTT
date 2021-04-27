#include <sys/types.h>
#include <unistd.h>

#include <algorithm>
#include <fstream>
#include <iostream>
#include <utility>
#include <vector>
using namespace std;
#define rep(i, n) for (int i = 0; i < (int)(n); i++)

string root_path = "";

void init(int argc, char *argv[]) {
    srand((unsigned)(time(NULL)));
    for (int i = 1; i < argc; i++) {
        root_path += std::string(argv[i]);
    }
}

// aとbをファイルストリームに出力する
// ファイル名は prefix_num.in (ex: 00_sample_00.in)
void output(int a, int b, const string &prefix, const int num) {
    char name[100];
    sprintf(name, "%s/testcase/%s_%02d.in", root_path.c_str(), prefix.c_str(),
            num);
    ofstream ofs(name);
    ofs << a << " " << b << endl;
    ofs.close();
}

int main(int argc, char *argv[]) {
    init(argc, argv);
    output(0, 1, "0_sample", 0);
}
