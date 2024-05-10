package com.emje.o324_gui

import android.os.Bundle

class MainActivity : TauriActivity() {
    companion object {
        init {
            System.loadLibrary("z")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Your other setup code here
    }
}
