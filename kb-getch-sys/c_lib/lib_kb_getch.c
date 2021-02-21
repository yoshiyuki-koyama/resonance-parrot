#include<conio.h>

char kb_getch(){
    if(_kbhit() != 0){
        return _getch();
    }
    return 0;
}