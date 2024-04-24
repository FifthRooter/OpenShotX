import sys
from PyQt5.QtWidgets import QApplication, QMainWindow

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("OpenShot X - Screen Capture Tool")
        self.setGeometry(100, 100, 600, 400)  # Set the dimensions of the window (x, y, width, height)

def main():
    app = QApplication(sys.argv)  # Create an application object
    window = MainWindow()         # Create a window object
    window.show()                 # Display the window
    sys.exit(app.exec_())         # Start the application's event loop

if __name__ == '__main__':
    main()
