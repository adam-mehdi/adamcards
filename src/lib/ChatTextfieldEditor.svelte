<script lang="ts">
    import { onMount, onDestroy } from 'svelte'
    import { Editor, nodePasteRule, textPasteRule, Extension } from '@tiptap/core'
    import StarterKit from '@tiptap/starter-kit'
    import { mergeAttributes } from '@tiptap/core'
    import Paragraph from '@tiptap/extension-paragraph';
    import Typography from '@tiptap/extension-typography'
    import Image from '@tiptap/extension-image'

    
  
    let element: HTMLElement
    export let index: number;
    export let content = "";

    let editor: Editor
    const ArrowReplacer = Extension.create({
        name: 'npspReplacer',
        addPasteRules() {
            return [
                textPasteRule({ find: / >>/g, replace: ' »' }),
                textPasteRule({ find: /(>*| )\* /g, replace: ' • ' }),
                textPasteRule({ find: /\%^\* /g, replace: ' • ' }),
            ]
        },
    })

    onMount(() => {
        editor = new Editor({
            element: element,
            extensions: [
                StarterKit,
                Typography,
                ArrowReplacer,
                Paragraph.extend({
                  parseHTML() {
                    return [{ tag: 'div' }]
                  },
                  renderHTML({ HTMLAttributes }) {
                    return ['div', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes), 0]
                  },
                }),
                Image.configure({
                  inline: true,
                  HTMLAttributes: {
                    class: 'rounded-lg object-contain max-h-64 w-full',
                  },

                })
            ],
            editorProps: {
                attributes: {
                    class: 'prose prose-light dark:prose-invert prose-md mx-auto focus:outline-none code:font-mono leading-6',
                },
                
            },
            content: content,
            editable: false,
            injectCSS: false,
            onTransaction: () => {
                editor = editor // force re-render so `editor.isActive` works as expected
            },
      })

      editor.on('update', ({ editor }) => {
        // The content has changed.
        content = editor.getHTML()
      })

      insertImages(content, editor);


    })
  
    onDestroy(() => {
      if (editor) {
        editor.destroy()
      }
    })
    

    function focusEditor() {
        editor.commands.focus()
    }

    function handlePaste(event: any) {
      const items = event.clipboardData.items
      for (let i = 0; i < items.length; i++) {
        const item = items[i]
        if (item.type.indexOf('image') === 0) {
          const file = item.getAsFile()
          if (file) {
            const reader = new FileReader()
            reader.onload = () => {
              const url = reader.result!.toString()
              // let image_id = invoke("create_image")
              // insert `[[[IMAGE ${image_id}]]]`
              editor.chain().focus().setImage({ src: url }).run()
            }
            reader.readAsDataURL(file)
          }
          event.preventDefault()
        }
    }
  }


   // This function parses and inserts <img> tags with data src as TipTap Image components
   const insertImages = (htmlContent: string, editor: any) => {
    const div = document.createElement('div');
    div.innerHTML = htmlContent;
    const images = div.querySelectorAll('img');

    images.forEach((img) => {
      const src = img.getAttribute('src');
      if (src!.startsWith('data:')) {
        editor.chain().focus().setImage({ src }).run();
      }
    });
  };


  </script>
  

  <div class="{index % 2 == 0 ? " bg-offwhite dark:bg-blacktext" : "font-light bg-platinum dark:bg-slate-700" } inline-block px-3 rounded-md p-2 m-2 mr-8 cursor-text focus-within:ring-2  ring-columbia transition-opacity duration-100">
    <div bind:this={element} class="mb-1" />
  </div>

  