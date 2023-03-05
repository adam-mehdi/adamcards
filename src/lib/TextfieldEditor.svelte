<script lang="ts">
    import { onMount, onDestroy } from 'svelte'
    import { Editor, nodePasteRule, textPasteRule, Extension } from '@tiptap/core'
    import StarterKit from '@tiptap/starter-kit'
    import { mergeAttributes } from '@tiptap/core'
    import Paragraph from '@tiptap/extension-paragraph';
    import Typography from '@tiptap/extension-typography'
    

  
    let element: HTMLElement

    export let is_textfield = false;
    export let is_gallery = false;
    export let is_answerbar = false;
    let min_height = is_textfield ? "200px" : "50px";
    min_height = is_gallery ? "25px" : min_height;
    let max_height = is_textfield ? "600px" : "300px"
    max_height = is_answerbar ? "100px" : max_height;
    min_height = is_answerbar ? "10px" : min_height;
    export let content = "" 
    export let autofocus = false

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

    })
  
    onDestroy(() => {
      if (editor) {
        editor.destroy()
      }
    })

    

    // function updateContent() {
    //     content = editor.getHTML()
    // }

    function focusEditor() {
        editor.commands.focus()
    }

  </script>
  
<div class="rounded-lg p-2 cursor-text focus-within:ring  ring-columbia transition-opacity duration-100" on:click={focusEditor} on:keydown={focusEditor}>
    <!-- on:input={updateContent}  -->
  <div 
    bind:this={element} 
    class="ProseMirror" 
    style="{!is_answerbar ? `max-height: ${max_height}` : ""}; min-height: {min_height}; overflow: {overflow}"
    />
</div>

  <!-- {#if editor}
    {editor.getHTML()}
  {/if} -->
  
  <style>


    .ProseMirror:focus {
        outline: none;
    }

  </style>