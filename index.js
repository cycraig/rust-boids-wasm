import init, { BoidFlock } from './pkg/boids.js';

const canvas = document.querySelector('canvas');
resize()
window.addEventListener('resize', resize);

const main = async () => {
  try {
    let wasm = await init();
    const memory = wasm.memory;
    
    const ctx = canvas.getContext('2d', { antialias: false });
    const flock = new BoidFlock(50);
    const count = flock.count();

    const positions = new Float32Array(memory.buffer, flock.positions(), 2*count);
    const velocities = new Float32Array(memory.buffer, flock.velocities(), 2*count);
    for(let i=0; i<count*2; i+=2) {
      positions[i] = Math.random()*canvas.width;
      positions[i+1] = Math.random()*canvas.height;
      velocities[i] = 2*Math.random()-1;
      velocities[i+1] = 2*Math.random()-1;
    }

    const renderLoop = () => {
      flock.set_width(canvas.width)
      flock.set_height(canvas.height)
      flock.update();

      // The memory locations change over time, so we need to refresh them in the loop.
      const positions = new Float32Array(memory.buffer, flock.positions(), 2*count);
      const velocities = new Float32Array(memory.buffer, flock.velocities(), 2*count);

      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = 'grey';
      ctx.strokeStyle = 'grey';
      
      for (let i = 0; i < count; i += 2) {
        const halfWidth = 2;
        const height = 8;
        
        // Calculate the heading of the boid.
        const angle = Math.atan2(velocities[i+1], velocities[i]);
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        
        // Draw a triangle.
        ctx.beginPath();
        ctx.moveTo(-sin*halfWidth + positions[i], cos*halfWidth + positions[i+1]);
        ctx.lineTo(sin*halfWidth + positions[i], -cos*halfWidth + positions[i+1]);
        ctx.lineTo(cos*height + positions[i], sin*height + positions[i+1]);
        ctx.lineTo(-sin*halfWidth + positions[i], cos*halfWidth + positions[i+1]);
        ctx.fill();
        ctx.stroke();
      }

      window.requestAnimationFrame(renderLoop);
    };

    renderLoop();
  } catch (error) {
    console.error("Error:", error);
  }
};

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}

main();
