// A simple script that modifies the entity's position to bob up and down
// position is a Vec3

// Since Rhai doesn't have a state persistence across frames built-in out of the box in this simple setup
// we can just make it move based on some logic, or use a sin wave based on time.
// Since we don't have total time exposed yet, we will just move it slightly on Y.

position.y += delta_time;

// Just bobbing up and down requires total time, let's just make it go up for now to prove it works.
