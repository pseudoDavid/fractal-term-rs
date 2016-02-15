extern crate rustbox;

use vector2::Vector2f;
use animator::{Spec, Animator};
use input::Command;
use textprinter::{TextPrinter, Asciifier};
use matrix::Matrix;
use mandelbrot;
use mandelbrot::Mandelbrot;


pub const CHARACTER_ASPECT_RATIO: f64 = 0.4;  // rough estimate of character aspect ratio    

const ZOOM_INCREMENT: f64 = 0.015;
const SMALL_ZOOM_INCREMENT: f64 = 0.003;
const CONTINUOUS_ZOOM: f64 = 0.010;
const VELOCITY_RATIO_INCREMENT: f64 = 0.005;
const TWEEN_MULTIPLIER: f64 = 0.08;
const FRICTION: f64 = 0.95;


pub struct App {
	
    matrix: Matrix<u16>,
    pub printer: TextPrinter,
    asciifier: Asciifier,
	
    mandelbrot: Mandelbrot,
	vp_center_anim: Animator<Vector2f>,
	vp_width_anim: Animator<f64>,

	view_width: usize,  // TODO: make 'pub fn set_dimensions(w: u16, h:16)'
	view_height: usize,
	max_escape: u16,
	
	count: u32,
}


impl App {
	
	pub fn new(view_width: usize, view_height: usize) -> App {
		
	    let max_esc = mandelbrot::DEFAULT_MAX_ESCAPE;
		
		App {
		    matrix: Matrix::new(view_width, view_height),
		    printer: TextPrinter::new(view_width, view_height),
		    asciifier: Asciifier::new(max_esc as f64),
		    
		    mandelbrot: Mandelbrot::new(CHARACTER_ASPECT_RATIO),
			vp_center_anim: Animator { value: Vector2f { x: 0.0, y: 0.0 }, spec: Spec::None },
			vp_width_anim: Animator { value: mandelbrot::DEFAULT_WIDTH, spec: Spec::None }, 
			
			view_width: view_width,
			view_height: view_height,
			max_escape: max_esc,
			count: 0
		}
    }
	
	pub fn handle_command(&mut self, command: &Command) {

		let vel_increment = self.vp_width_anim.value as f64 * VELOCITY_RATIO_INCREMENT;  // abstract this

		match *command {
			
			Command::PositionVelocity(xm, ym) => {  
				let increment = Vector2f { x: vel_increment * xm, y: vel_increment * ym };
				match self.vp_center_anim.spec {
					Spec::Velocity { ref mut velocity, .. } => {
						*velocity = *velocity + increment;
					},
					_ => self.vp_center_anim.spec = Spec::Velocity { velocity: increment, friction: FRICTION },
				}
			},

			Command::PositionTween(char_col, char_row) => {  // rem, these params are 0-indexed not 1-indexed

				let half_screen_w = self.view_width as f64 / 2.0;
				let ratio_x = (char_col as f64 - half_screen_w) / half_screen_w;
				let half_vp_width = self.vp_width_anim.value / 2.0;
				let target_x = self.vp_center_anim.value.x + (ratio_x * half_vp_width);
				
				// y requires extra logic:
				let ar = self.view_width as f64 / self.view_height as f64;
				let viewport_height = self.vp_width_anim.value * (1.0 / ar)  *  (1.0 / self.mandelbrot.element_aspect_ratio);

				let half_screen_h = self.view_height as f64 / 2.0;
				let ratio_y = (char_row as f64 - half_screen_h) / half_screen_h;
				let half_vp_height = viewport_height / 2.0;
				let target_y = self.vp_center_anim.value.y + (ratio_y * half_vp_height);

				self.vp_center_anim.spec = Spec::Tween {
					target: Vector2f { x: target_x, y: target_y }, coefficient: TWEEN_MULTIPLIER}					
			}

			Command::Zoom(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				let current_scale = match self.vp_width_anim.spec {
					Spec::Scale { scale, .. } => scale,
					_ => 0.0,
				};
				self.vp_width_anim.spec = Spec::Scale { scale: current_scale + increment, friction: FRICTION };
			},
			
			Command::ZoomContinuous(multiplier) => {
				let increment = ZOOM_INCREMENT * multiplier;
				self.vp_width_anim.spec = Spec::Scale { scale: increment, friction: 1.0 };
			},

			Command::Resize(w, h) => {
				// TODO
			}
						
			Command::Stop => { 
				self.vp_center_anim.spec = Spec::None;
				self.vp_width_anim.spec = Spec::None; 
			},
			Command::Reset => { 
				self.vp_center_anim.value.x = 0.0;
				self.vp_center_anim.value.y = 0.0;
				self.vp_center_anim.spec = Spec::None;
				self.vp_width_anim.value = mandelbrot::DEFAULT_WIDTH;
				self.vp_width_anim.spec = Spec::None; 
			},

			_ => {}
		}
	}
	
	pub fn get_magnification(&self) -> f64 {
		mandelbrot::DEFAULT_WIDTH / self.vp_width_anim.value
	}
	
	pub fn update(&mut self) {
		self.vp_width_anim.update();
		self.vp_center_anim.update();
	}
	
	pub fn calculate(&mut self) {
        self.mandelbrot.write_matrix(self.vp_center_anim.value.clone(), self.vp_width_anim.value, &mut self.matrix);
	}
	
	pub fn draw_frame(&mut self, debug_info: &String) {
        
        self.printer.draw_ascii_rect(&self.matrix, &self.asciifier);
    
        let info = format!(" {:.0}x {}", self.get_magnification(), debug_info);    
        self.printer.draw_string(&info, 1, self.view_height - 1);

        if true && self.count % 60 < 15 {
        	// show center-point
        	let x =  self.view_width / 2;
        	let y = self.view_height / 2;
	        self.printer.draw_string(&"█".to_string(), x,y);	        	
        }
        
        self.printer.render();
        
        self.count += 1;
	}
}
