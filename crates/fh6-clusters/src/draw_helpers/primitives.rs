use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

pub fn fill_rect(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, rgba: Color) {
    if let Some(r)=Rect::from_xywh(x,y,w,h){
        let mut p=Paint::default(); p.set_color(rgba);
        pixmap.fill_rect(r,&p,Transform::identity(),None);
    }
}
pub fn fill_rect_rounded(pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32, radius: f32, rgba: Color) {
    // simple rounded via path
    let mut pb=PathBuilder::new();
    let r=radius.max(0.0).min(w.min(h)/2.0);
    let x1=x+r; let y1=y+r; let x2=x+w-r; let y2=y+h-r;
    pb.move_to(x+r,y);
    pb.line_to(x2,y);
    pb.quad_to(x+w,y,x+w,y+r);
    pb.line_to(x+w,y2);
    pb.quad_to(x+w,y+h,x2,y+h);
    pb.line_to(x1,y+h);
    pb.quad_to(x,y+h,x,y2);
    pb.line_to(x,y1);
    pb.quad_to(x,y,x+r,y);
    if let Some(p)=pb.finish(){
        let mut paint=Paint::default(); paint.set_color(rgba);
        pixmap.fill_path(&p,&paint,Transform::identity(),None);
    }
}
pub fn stroke_polyline(pixmap: &mut PixmapMut, points:&[(f32,f32)], width:f32, rgba:Color, closed:bool){
    if points.len()<2 {return;}
    let mut pb=PathBuilder::new();
    pb.move_to(points[0].0,points[0].1);
    for p in points.iter().skip(1){ pb.line_to(p.0,p.1); }
    if closed { pb.close(); }
    if let Some(p)=pb.finish(){
        let mut paint=Paint::default(); paint.set_color(rgba);
        let stroke=Stroke{width, ..Default::default()};
        pixmap.stroke_path(&p,&paint,&stroke,Transform::identity(),None);
    }
}
pub fn fill_polygon(pixmap:&mut PixmapMut, points:&[(f32,f32)], rgba:Color){
    if points.len()<3 {return;}
    let mut pb=PathBuilder::new();
    pb.move_to(points[0].0,points[0].1);
    for p in points.iter().skip(1){ pb.line_to(p.0,p.1); }
    pb.close();
    if let Some(p)=pb.finish(){
        let mut paint=Paint::default(); paint.set_color(rgba);
        pixmap.fill_path(&p,&paint,Transform::identity(),None);
    }
}
pub fn stroke_arc_deg(pixmap:&mut PixmapMut, cx:f32, cy:f32, r:f32, start_deg:f32, sweep_deg:f32, width:f32, rgba:Color){
    let steps=((sweep_deg.abs()*0.5).max(2.0)) as usize;
    let step=sweep_deg/steps as f32;
    let mut pts=Vec::with_capacity(steps+1);
    for i in 0..=steps{
        let a=(start_deg + step*i).to_radians();
        pts.push((cx + r*a.cos(), cy + r*a.sin()));
    }
    stroke_polyline(pixmap,&pts,width,rgba,false);
}
pub fn draw_soft_glow(pixmap:&mut PixmapMut, cx:f32, cy:f32, rx:f32, ry:f32, rgba:Color){
    // simple ellipse fill
    fill_rect(pixmap, cx-rx, cy-ry, rx*2.0, ry*2.0, rgba);
}
