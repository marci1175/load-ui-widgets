local output = ui_textedit()

function hellowrld()
    print(output:get_buffer())
end

ui_separator()

ui_button("Print text!", hellowrld)

