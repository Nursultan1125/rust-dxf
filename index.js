
import init, {sli_to_json} from './pkg/dxf_reader.js';

document.getElementById('fileInput').addEventListener('change', async (event) => {
    const file = event.target.files[0];
    const text = await file.text();  // Read DXF file content

    await init();  // Initialize WASM module
    const json = sli_to_json(text);
    // const json1 = dxf_to_json(text);
    console.log(JSON.parse(json))

    document.getElementById('output').textContent = json;
});