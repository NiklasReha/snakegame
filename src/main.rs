use rand::Rng;
use crossterm::terminal;
use std::io;
use std::io::Read;
use crossbeam_channel::unbounded;
use std::{
    thread,
    time,
};
use std::io::stdout;
use crossterm::{QueueableCommand};
use std::io::{Write};
use win32console::console::WinConsole;

fn main() {
loop{
//Abfrage der Spielfeldparameter un initalisieren des Feldes
    let _clean_up = CleanUp;
    let weite =20;
    let hoehe = 20;
    let mut playingfield:Vec<Vec<i32>> = Vec::new();
    println!("Press ENTER to start");
    let mut buf = [0; 1];
    io::stdin().read(&mut buf).expect("Failed to read line");

    for i in 0..hoehe+2{
        playingfield.push(Vec::new());
        for _d in 0..weite+2{
            playingfield[i as usize].push(0);
        }
    }

//Erstellen der Anfangsschlange und des ersten Foods

    let mut head =Snakepoint{pos_x:weite/2, pos_y:hoehe/2,length:3,previous_point:None};
    let mut food=Food{pos_x:16,pos_y:17};
    let mut std=io::stdin();
    WinConsole::output().clear().expect("Irgendwas lief falsch");
    let mut direction=MoveDirection{vec_y:0,vec_x:-1};
    terminal::enable_raw_mode().expect("Could not turn on Raw mode");
    let (s, r) = unbounded::<Result<MoveDirection, MoveDirection>>();
    let (s2, r2) = unbounded::<Result<bool, bool>>();
    let (s3, r3) = unbounded::<Result<bool, bool>>();
    
    //Fängt getätigte Inputs ab
    let _child_handle = thread::spawn(move|| {
        let mut direction_thread = MoveDirection{vec_y:0,vec_x:-1};
    
        loop{
            match r2.try_recv(){
                Ok(x)=>{
                    match x{
                        Ok(z)=>{if z{
                            thread::sleep(time::Duration::from_millis(80));
                            let mut buf = [0; 1];
                            std.read(&mut buf).expect("Failed to read line");
                            s3.send(Ok(buf[0] as char =='y'));
                            break}},
                        Err(_u)=>{}
                    }},
                Err(_d)=>{}
            };   
            let mut buf = [0; 1];
            std.read(&mut buf).expect("Failed to read line");    
            if buf[0] as char =='w' && direction_thread.vec_y !=1 {
                direction_thread.vec_x=0;
                direction_thread.vec_y=-1;
            }
            else if buf[0] as char =='s' && direction_thread.vec_y !=-1{
                direction_thread.vec_x=0;
                direction_thread.vec_y=1;
            }
            else if buf[0] as char =='a'&& direction_thread.vec_x != 1{
                direction_thread.vec_x=-1;
                direction_thread.vec_y=0;
            }
            else if buf[0] as char =='d' && direction_thread.vec_x !=-1{
                direction_thread.vec_x=1;
                direction_thread.vec_y=0;
            }      
            else if buf[0] as char =='q'{
                WinConsole::output().clear().expect("Irgendwas lief falsch");
                terminal::disable_raw_mode().expect("Could not disable raw mode");
                stdout().queue(crossterm::cursor::Show).expect("Irgendwas lief falsch");
                std::process::exit(0);
            }  
            s.send(Ok(direction_thread.clone())).unwrap();
        }
    });

    WinConsole::output().clear().expect("Irgendwas lief falsch");

    //Gameloop
    loop{  
        match r.try_recv(){
            Ok(x)=>{
                match x{
                    Ok(z)=>{direction=z},
                    Err(_u)=>{}
                }},
            Err(_d)=>{}
        };       
        let mut stdout = stdout();
        stdout.queue(crossterm::cursor::Hide).expect("Irgendwas lief falsch");
        stdout.queue(crossterm::cursor::MoveTo(0,0)).expect("Irgendwas lief falsch");
        let eaten=head.move_snek(direction.clone(), &mut food, &mut playingfield);
        if head.draw_snake(&mut playingfield, &mut food,eaten){
            break
        }
        let mut ausgabe=String::new();
        ausgabe+=&"  ".to_string();
        for _d in 1..playingfield[0].len()-2{
            ausgabe+=&"___".to_string();
        }
        ausgabe+=&"\n ".to_string();
        for i in 1..playingfield.len()-2{
            ausgabe+=&"|".to_string();
            for d in 1..playingfield[0].len()-2{
                if playingfield[i as usize][d as usize] == 1{
                    ausgabe+=&" O ".to_string();
                }
                else if playingfield[i as usize][d as usize] == 2{
                    ausgabe+=&" 6 ".to_string();
                }
                else{
                    ausgabe+=&"   ".to_string();
                }
            }
            ausgabe+=&"|\n ".to_string();
        }
        ausgabe+=&"|".to_string();
        for _d in 1..playingfield[0].len()-2{
            ausgabe+=&"___".to_string();
        }
        ausgabe+=&"|\n\n ".to_string();
        ausgabe+=&"Score: ".to_string();
        ausgabe+=&(head.length-3).to_string();
        stdout.write(format!("{}", ausgabe).as_bytes()).expect("Irgendwas lief falsch");
        thread::sleep(time::Duration::from_millis(110));
        
    }
    let mut stdout=std::io::stdout();
    s2.send(Ok(true)); 
    stdout.queue(crossterm::cursor::Show).expect("Irgendwas lief falsch");
    WinConsole::output().clear().expect("Irgendwas lief falsch");
    println!("You lost!  |:<(~) Noooooo");
    thread::sleep(time::Duration::from_millis(150));
    println!("Press \"y\" to continue");
    match r3.recv(){
        Ok(x)=>{
            match x{
                Ok(z)=>{if !z {
                    break;
                }},
                Err(_u)=>{}
            }},
        Err(_d)=>{}
    };
    
    terminal::disable_raw_mode().expect("Could not disable raw mode");
}
}

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        let mut stdout=std::io::stdout();
        stdout.queue(crossterm::cursor::Show).expect("Irgendwas lief falsch");
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        WinConsole::output().clear().expect("Irgendwas lief falsch");
    }
}

#[derive(Clone)]
pub struct Snakepoint{
    length:i32,
    pos_x:i32,
    pos_y:i32,
    previous_point: Option<Box<Snakepoint>>,
}


#[derive(Clone)]
pub struct MoveDirection{
    vec_x:i32,
    vec_y:i32
}

impl Snakepoint{

    pub fn detect_collision(&self,weite:i32,hoehe:i32)->bool{

        if self.pos_x < 1 || self.pos_x > weite-1 || self.pos_y < 1 || self.pos_y > hoehe-1{
            return true;
        }
        return false;
    }

    pub fn move_snek(&mut self,direction:MoveDirection,food:&mut Food,playingfield:&mut Vec<Vec<i32>>)->bool{
        let mut substitute=self.clone();
        let hoehe=playingfield.len()-1;
        let weite=playingfield[0].len()-1;
        let mut eaten=false;
        for i in 0..hoehe{
            for d in 0..weite{
                playingfield[i as usize][d as usize]=0;
            }
        }
        let mut new_head=Snakepoint{previous_point:None,pos_x:direction.vec_x+self.pos_x,pos_y:direction.vec_y+self.pos_y,length:self.length};
        playingfield[new_head.pos_y as usize][new_head.pos_x as usize] =1;
        if new_head.pos_x ==food.pos_x && new_head.pos_y ==food.pos_y{
            new_head.length+=1;
            eaten = true;
        }
        new_head.previous_point=substitute.rm_unneeded_and_update(new_head.length);
        *self=new_head;
        return eaten;
    }

    pub fn draw_snake(&self,playingfield:&mut Vec<Vec<i32>>,food:&mut Food,eaten:bool)->bool{
        let mut iterator=self.clone();
        playingfield[iterator.pos_y as usize][iterator.pos_x as usize]=1;
        loop{
            match iterator.previous_point{
                Some(x)=>{iterator=*x;
                    if playingfield[iterator.pos_y as usize][iterator.pos_x as usize] !=1{
                    playingfield[iterator.pos_y as usize][iterator.pos_x as usize]=1;
                }else{
                    return true
                }},
                None=>break
            }
        }
        if eaten{
            food.respawn(playingfield.clone());
        }
        playingfield[food.pos_y as usize][food.pos_x as usize]=2;
        return self.detect_collision((playingfield.len()-2) as i32,(playingfield[0].len()-2) as i32);
    }

    pub fn rm_unneeded_and_update(&mut self,length:i32)->Option<Box<Snakepoint>>{
        self.length = length-1;
        if self.length > 0{
        let mut _safe=self.clone();
        let mut iterator=self.clone();
        match iterator.previous_point{
            Some(x)=>{
                    _safe=*x.clone();                  
            },
            None=>{return Some(Box::new(iterator));}
        }
        iterator.previous_point=_safe.rm_unneeded_and_update(self.length);
        return Some(Box::new(iterator));
    }
    else{
        return None;
    }
    }


}

#[derive(Clone, Copy)]
pub struct Food{
    pos_x:i32,
    pos_y:i32
}

impl Food{
    pub fn respawn(&mut self,playingfield:Vec<Vec<i32>>){
        let mut rng = rand::thread_rng();
        loop{
        self.pos_x =rng.gen_range(1..(playingfield[0].len()-2)as i32);
        self.pos_y=rng.gen_range(1..(playingfield.len()-2)as i32);
            if playingfield[self.pos_y as usize][self.pos_x as usize] != 1 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod eval {
use super::*;

#[test]
pub fn move_up_single_part(){
    let mut head=Snakepoint{pos_x:1,pos_y:1,length:1,previous_point:None};
    let mut food=Food{pos_x:0,pos_y:0};
    let mut playingfield:Vec<Vec<i32>> = Vec::new();
    playingfield.push(Vec::new());
    playingfield.push(Vec::new());
    playingfield[0 as usize].push(0);
    playingfield[0 as usize].push(0);
    playingfield[1 as usize].push(0);
    playingfield[1 as usize].push(0);
    head.move_snek(MoveDirection{vec_y:-1,vec_x:0},&mut food,&mut playingfield);
    assert_eq!(head.pos_y,0);
    assert_eq!(head.pos_x,1);
}

#[test]
pub fn move_up_multiple_part(){
    let mut head=Snakepoint{pos_x:1,pos_y:1,length:2,previous_point:None};
    let mut playingfield:Vec<Vec<i32>> = Vec::new();
    playingfield.push(Vec::new());
    playingfield.push(Vec::new());
    playingfield.push(Vec::new());
    playingfield.push(Vec::new());
    playingfield[0 as usize].push(0);
    playingfield[0 as usize].push(0);
    playingfield[1 as usize].push(0);
    playingfield[1 as usize].push(0);
    playingfield[2 as usize].push(0);
    playingfield[2 as usize].push(0);
    head.move_snek(MoveDirection{vec_y:-1,vec_x:0},&mut Food{pos_x:0,pos_y:0},&mut playingfield);
    match head.previous_point{
        Some(x)=>{assert_eq!(x.pos_y,1);
            assert_eq!(x.pos_x,1);
            assert_eq!(x.length,1);
                match x.previous_point{
                    Some(_x)=>{assert_eq!(false,true)},
                    None=>{}
                }},
        None=>{assert_eq!(false,true);},
    };
    assert_eq!(head.pos_y,0);
    assert_eq!(head.pos_x,1);
    assert_eq!(head.length,2);


}

#[test]
pub fn detect_collision_true(){
    let head=Snakepoint{pos_x:1,pos_y:1,length:2,previous_point:Some(Box::new(Snakepoint{previous_point:None,pos_x:1,pos_y:1,length:1}))};
    assert_eq!(head.detect_collision(20,20),true);
}

#[test]
pub fn detect_collision_false(){
    let head=Snakepoint{pos_x:1,pos_y:1,length:2,previous_point:Some(Box::new(Snakepoint{previous_point:None,pos_x:2,pos_y:1,length:1}))};
    assert_eq!(head.detect_collision(20,20),false);
}

#[test]
pub fn detect_collision_bounds_true(){
    let head=Snakepoint{pos_x:1,pos_y:1,length:2,previous_point:Some(Box::new(Snakepoint{previous_point:None,pos_x:1,pos_y:2,length:1}))};
    assert_eq!(head.detect_collision(1,1),true);
}

}

