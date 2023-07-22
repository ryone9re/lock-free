use std::arch::asm;
use std::cell::UnsafeCell;
use std::ptr::null_mut;

// スタックノード､リスト構造で管理
#[repr(C)]
struct Node<T> {
    next: *mut Node<T>,
    data: T,
}

// スタックの先頭
#[repr(C)]
pub struct StackHead<T> {
    head: *mut Node<T>,
}

impl<T> StackHead<T> {
    fn new() -> Self {
        StackHead { head: null_mut() }
    }

    pub fn push(&mut self, v: T) {
        // 追加するノードを作成
        let node = Box::new(Node {
            next: null_mut(),
            data: v,
        });

        // Box型の値からポインタを取り出す
        let ptr = Box::into_raw(node) as *mut u8 as usize;

        // ポインタのポインタを取得
        // headの格納されているメモリをLL/SC
        let head = &mut self.head as *mut *mut Node<T> as *mut u8 as usize;

        // LL/SCを用いたpush
        unsafe {
            asm!(
                "1:
                ldxr {next}, [{head}] // next = *head
                str {next}, [{ptr}] // *ptr = next
                stlxr w10, {ptr}, [{head}] // *head = ptr
                // if tmp != 0 then goto 1
                cbnz w10, 1b",
                next = out(reg) _,
                ptr = in(reg) ptr,
                head = in(reg) head,
                out("w10") _
            )
        };
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            // ポインタのポインタを取得
            // headの格納されているメモリをLL/SC
            let head = &mut self.head as *mut *mut Node<T> as *mut u8 as usize;

            // popしたノードへのアドレスを格納
            let mut result: usize;

            // LL/SCを用いたpop
            asm!(
                "1:
                ldaxr {result}, [{head}] // result = *head
                // if result != NULL then goto 2
                cbnz {result}, 2f

                // if NULL
                clrex // clear exclusive
                b 3f // goto 3

                // if not NULL
                2:
                ldr {next}, [{result}] // next = *result
                stxr w10, {next}, [{head}] // *head = next
                // if tmp != 0 then goto 1
                cbnz w10, 1b

                3:",
                next = out(reg) _,
                result = out(reg) result,
                head = in(reg) head,
                out("w10") _
            );

            if result == 0 {
                return None;
            }

            // ポインタをBoxに戻して､中の値をリターン
            let ptr = result as *mut u8 as *mut Node<T>;
            let head = Box::from_raw(ptr);
            Some(head.data)
        }
    }
}

impl<T> Drop for StackHead<T> {
    fn drop(&mut self) {
        // データ削除
        let mut node = self.head;
        while !node.is_null() {
            // ポインタをBoxに戻す操作を繰り返す
            let n = unsafe { Box::from_raw(node) };
            node = n.next;
        }
    }
}

// StackHeadをUnsafeCellで保存するのみ
pub struct Stack<T> {
    data: UnsafeCell<StackHead<T>>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            data: UnsafeCell::new(StackHead::new()),
        }
    }

    pub fn get_mut(&self) -> &mut StackHead<T> {
        unsafe { &mut *self.data.get() }
    }
}

// スレッド間のデータ共有と､チャネルを使った送受信が可能と設定
unsafe impl<T> Sync for Stack<T> {}
unsafe impl<T> Send for Stack<T> {}
