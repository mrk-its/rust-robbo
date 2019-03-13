import { Universe, Cell } from "rust-robbo";
import loadImage from 'image-promise';
import { AssertionError } from "assert";

// Construct the universe, and get its width and height.

// Give the canvas room for all of our cells and a 1px border
// around each of them.


function sleep(delay) {
  return (new Promise((resolve) => setTimeout(resolve, delay)))
}

function get_image_data(image) {
  var canvas = document.getElementById('offscreen-canvas');
  var context = canvas.getContext('2d')
  context.fillStyle = '#608050'
  context.fillRect(0, 0, image.width, image.height)
  context.drawImage(image, 0, 0 )
  return context.getImageData(0, 0, image.width, image.height)
}

loadImage(['data/skins/original/icons32.png']).then(function(images) {
  sleep(100).then(function() { // TODO
    function get_current_level() {
      return localStorage.current_level && parseInt(localStorage.current_level) || 0
    }

    function store_current_level(level) {
      localStorage.current_level = level
    }

    const image_data = get_image_data(images[0]);

    const universe = Universe.new(image_data, get_current_level())
    const inventory = document.getElementById("inventory")
    const canvas = document.getElementById("robbo-canvas")
    let ctx = canvas.getContext('2d');
    function renderLoop() {
        if(canvas.width != universe.width() || canvas.height != universe.height()) {
          canvas.width = universe.width()
          canvas.height = universe.height()
          ctx = canvas.getContext('2d');
        }
        let current_level = universe.current_level()
        if(current_level != get_current_level()) store_current_level(current_level);
        universe.draw(ctx);
        inventory.textContent = universe.get_inventory();
        requestAnimationFrame(renderLoop);
      };
    requestAnimationFrame(renderLoop);
    function key_handler(event) {
      // console.log(event);
      var handled = universe.on_keyboard_event(event, event.type == "keydown");
      if(handled) {
        event.preventDefault();
      }
    }
    document.addEventListener('keydown', key_handler);
    document.addEventListener('keyup', key_handler);
    universe.draw(ctx);
  })
})
