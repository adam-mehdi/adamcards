<script lang="ts">
    import { onMount, onDestroy } from 'svelte'
    import { Editor, nodePasteRule, textPasteRule, Extension } from '@tiptap/core'
    import StarterKit from '@tiptap/starter-kit'
    import { mergeAttributes } from '@tiptap/core'
    import Paragraph from '@tiptap/extension-paragraph';
    import Typography from '@tiptap/extension-typography'
    import Image from '@tiptap/extension-image'

    
  
    let element: HTMLElement
    
    export let is_textfield = false;
    export let is_gallery = false;
    export let is_answerbar = false;
    let min_height = is_textfield ? "200px" : "75px";
    let max_height = is_textfield ? "600px" : "300px"
    max_height = is_answerbar ? "100px" : max_height;
    min_height = is_answerbar ? "10px" : min_height;
    export let content = "" 
    export let autofocus = false

    if (is_gallery) {
      max_height = "140px"
      min_height = "140px"
    }


    export let is_expl = false
    if (is_expl) {
      max_height = "120px"
      min_height = "120px"

    }

    let overflow = is_answerbar ? "scroll" : "scroll" 



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
    
    export let loading = false;

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
                    class: 'prose dark:prose-invert prose-md mx-auto focus:outline-none font-sans code:font-mono leading-7',
                },
                
            },
            content: content,
            autofocus: autofocus,
            editable: true,
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
  
{#if !loading}
<div class="rounded-lg p-2 cursor-text focus-within:ring-2  ring-columbia transition-opacity duration-100" on:click={focusEditor} on:keydown={focusEditor}>
  <div 
    bind:this={element} 
    class="ProseMirror" 
    style="{!is_answerbar ? `max-height: ${max_height};` : "padding-right: 23px;"}; min-height: {min_height}; overflow: {overflow}"
    on:paste={handlePaste} 
    />
</div>

{:else}
<div class="rounded-lg p-2 cursor-text focus-within:ring-2  ring-columbia transition-opacity duration-100" on:click={focusEditor} on:keydown={focusEditor}>
  <div style="{!is_answerbar ? `max-height: ${max_height};` : "padding-right: 23px;"}; min-height: {min_height}; overflow: {overflow}">
    {content}
  </div>
</div>

{/if}

  <!-- {#if editor}
    {editor.getHTML()}
  {/if} -->

  <style>


    .ProseMirror:focus {
        outline: none;
    }


  </style>