/* Existing import statements */
import copy from 'rollup-plugin-copy'

/* Skip to the export statement, `plugins` item and add `copy`*/
export default {
  /* Existing key: values... */
  plugins: [
    copy({
      targets: [
        { src: 'node_modules/tinymce/*', dest: 'public/tinymce' }
      ]
    }),
    /* More existing configuration... */
  ]
}
