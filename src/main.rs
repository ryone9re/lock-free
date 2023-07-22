use std::sync::Arc;

use stack::Stack;

mod stack;

const NUM_LOOP: usize = 1000000; // ループ回数
const NUM_THREADS: usize = 4; // スレッド数

fn main() {
    let stack = Arc::new(Stack::<usize>::new());
    let mut v = Vec::new();

    for i in 0..NUM_THREADS {
        let stack0 = stack.clone();
        let t = std::thread::spawn(move || {
            if i & 1 == 0 {
                // 偶数スレッドはpush
                for j in 0..NUM_LOOP {
                    let k = i * NUM_LOOP + j;
                    stack0.get_mut().push(k);
                    println!("push: {k}");
                }
            } else {
                // 奇数スレッドはpop
                for _ in 0..NUM_LOOP {
                    loop {
                        // pop, Noneの場合やり直し
                        if let Some(k) = stack0.get_mut().pop() {
                            println!("pop: {k}");
                            break;
                        }
                    }
                }
                println!("finished pop: #{i}");
            }
        });

        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }

    assert!(stack.get_mut().pop().is_none());
}
