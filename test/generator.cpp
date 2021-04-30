#include <fstream>
#include <iostream>
using namespace std;

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
    /* ここから書き換える */
    ofs << a << " " << b << endl;
    /* ここまで */
    ofs.close();
}

int main(int argc, char *argv[]) {
    init(argc, argv);
    /* ここから書き換える */
    output(0, 0, "0_sample", 0);
    output(0, 1, "0_sample", 1);
    output(1, 0, "0_sample", 2);
    output(1, 1, "0_sample", 3);
    /* ここまで */
}
