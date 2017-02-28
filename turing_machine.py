#!/usr/bin/env PYTHON3

from collections import namedtuple

"""
An IntMachine contains two integer variables: one active and one inactive.
It also contains a stack of integeres. The stack is represented by a list, where
the end of the list (high indices) is the top of the stack, and the beginning of the
list is bottom of the stack. In other words, popping gets you the last variable in
the list. Only the active variable can be modified.

This model is meant to represent the capabilities of rocketlang, as defined here:
https://github.com/PieMan2201/rocketlang
"""
class IntMachine:

    def __init__(self):
        self.active_var = 0
        self.inactive_var = 0
        self.stack = []

    """
    Sets the active variable to the given character
    "Take the shot!" from rocketlang
    """
    def set_active_var(self, char):
        if ord(c) >= 128:
            raise TypeError("This machine only supports basic ASCII [0, 127]")
        self.active_var = ord(char)

    """
    Increments the active variable
    "Wow!" from rocketlang
    """
    def incr_var(self):
        self.active_var += 1

    """
    Decrements the active variable
    "Close One!" from rocketlang
    """
    def decr_var(self):
        self.active_var -= 1

    """
    Copies the active variable to the inactive variable
    "Whoops..." from rocketlang
    """
    def save_active_var(self):
        self.inactive_var = self.active_var

    """
    Swaps the values of the active and inactive variables
    "OMG!" from rocketlang
    """
    def swap(self):
        temp = self.active_var
        self.active_var = self.inactive_var
        self.inactive_var = temp

    """
    Pushes the value 0 onto the stack
    "Noooo!" from rocketlang
    """
    def push_0(self):
        self.stack.append(0)

    """
    Pops the item at the top of the stack, and sets active_var equal to that item
    "Centering..." from rocketlang
    """
    def pop_to_active_var(self):
        active_var = stack.pop()

    """
    Pushes the value of the active variable onto the stack
    "Defending..." from rocketlang
    """
    def push_active_var(self):
        stack.append(active_var)

    """
    Runs a single if statement that only executes if the active variable is equal to the inactive
    variable. Each given function is run in order, and must take self as a singular argument to
    enforce that it is a function of this machine.
    "Nice shot!" and "What a save!" from rocketlang - we could require the user to end every
    function list with  "What a save!" equivalent to preserve better correlation with rocketlang,
    but I think this shortcut is okay to take.
    """
    def if_statement(self, funcs):
        if self.active_var == self.inactive_var:
            for func in funcs:
                func(self)

    """
    Runs a loop that operates the given functions in order while the active variable is >0.
    Each function is assumed to only take self as an argument, to enforce that they are all
    functions of this machine
    "Great pass!" and "Thanks!" from rocketlang - we could require the user to end every function
    list with a "Thanks!" equivalent to preserve better correlation with rocketlang, but I think
    this shortcut is okay to take.
    """
    def loop(self, funcs):
        while self.active_var > 0:
            for func in funcs:
                func(self)


"""
An expanded version of IntMachine that adds convenience functions, but only uses IntMachine's
functions to do it, so it doesn't add any more functionality.
"""
class ExpandedMachine(IntMachine):

    """
    Resets the value of the active variable to 0
    """
    def reset_active_var(self):
        self.push_0()  # Push 0 onto the stack
        self.pop_to_active_var()  # Pop 0 from the stack into the active variable

    """
    Writes the given string to the stack, as integer values. The last character in the string
    will become the value at the top of the stack.
    """
    def write_string_to_stack(self, s):
        for c in s:
            self.set_active_var(c)
            self.push_active_var()


Instruction = namedtuple('Instruction', '')

class TuringMachine(ExpandedMachine):

    def __init__(self, *rules):
        super(TuringMachine, self).__init__()
        self.rules = rules


def main():
    pass


if __name__ == '__main__':
    main()
