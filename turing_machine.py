#!/usr/bin/env PYTHON3

from collections import namedtuple


class IntMachine:
    """
    An IntMachine contains two integer variables: one active and one inactive.
    It also contains a stack of integeres. The stack is represented by a list, where
    the end of the list (high indices) is the top of the stack, and the beginning of the
    list is bottom of the stack. In other words, popping gets you the last variable in
    the list. Only the active variable can be modified.

    This model is meant to represent the capabilities of rocketlang, as defined here:
    https://github.com/PieMan2201/rocketlang
    """

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


class ExpandedMachine(IntMachine):
    """
    An expanded version of IntMachine that adds convenience functions, but only uses IntMachine's
    functions to do it, so it doesn't add any more functionality.
    """

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


"""
State is the name of the state
"""
Condition = namedtuple('Condition', 'state tape_symbol')

"""
action is one of 'L R W' meaning move left, move right, or write to tape
new_symbol is the symbol to write (irrelevant if aciton isn't 'W')
next_state is the name of the next state to move to
"""
Action = namedtuple('Action', 'function next_state')


class Language:

    def __init__(self, func, empty_symbol):
        self.func = func
        self.empty_symbol = empty_symbol

        # Make sure the empty symbol isn't the language, to avoid confusion
        if self.contains(empty_symbol):
            raise ValueError("Empty symbol cannot be in the language")

    def contains(self, symbol):
        return self.func(symbol)

ASCII = Language(lambda c: ord(c) < 128, 'ε')


class TuringMachine(ExpandedMachine):

    def __init__(self, accepting_states, initial_state, language=ASCII):
        self.accepting_states = set(accepting_states)
        self.language = language
        self.initial_state = initial_state
        self.instructions = dict()

    """
    Checks if the given symbol is in this machine's language. If not, raises a ValueError.
    Also raises a ValueError if the length of the supposed symbol is not 1.
    """
    def check_symbol(self, symbol):
        if len(symbol) != 1:
            raise ValueError("Not a single character: " + symbol)
        if not self.language.contains(symbol):
            raise ValueError("Symbol not in language: " + symbol)

    """
    Adds the given condition/action pair to this machine's instruction set.
    """
    def add_instruction(self, condition, action):
        # Check the validity of both symbols
        self.check_symbol(condition.tape_symbol)
        if action.new_symbol:
            self.check_symbol(action.new_symbol)

        # TODO check vailidity of action.function here
        self.instructions[condition] = action  # Add it to the instruction set

    def apply_action(self, action):
        action.function()  # Run the function (left, right, or write)
        self.set_state(action.next_state)  # Go to next state

    def move_left(self):
        self.head -= 1
        # If we hit the left end, extend the tape with an empty square
        if self.head < 0:
            self.tape.insert(0, self.language.empty_symbol)

    def move_right(self):
        self.head += 1
        # If we hit the right end, extend the tape with an empty square
        if self.head >= len(self.tape):
            self.tape.append(self.language.empty_symbol)

    def write_symbol(self, symbol):
        self.tape[self.head] = symbol

    def set_state(self, next_state):
        self.state = next_state

    """
    Run this machine on the given string, returning True if it is in the language, false if not.
    """
    def run(self, s):
        # Initialize values for this run
        self.tape = []
        self.head = 0
        self.state = self.initial_state

        # Fill the tape, checking that each symbol is in the language while we're at it
        for symbol in s:
            self.check_symbol(symbol)
            self.tape.append(symbol)

        # Loop until we reach an accepting state
        while self.state not in self.accepting_states:
            # Try to find an action for this state/symbol pair. If there is none, halt
            try:
                action = self.instructions[Condition(self.state, self.tape[self.head])]
            except KeyError:
                # No instruction, halt and return false
                return False
            self.apply_action(action)
        return True

    """
    Run this machine and print ACCEPT or REJECT and the machine's state at halt time.
    """
    def run_and_print(self, s):
        result = self.run(s)
        print(self)
        print("ACCEPT" if result else "REJECT")
        print("")

    def __str__(self):
        tape = ''.join(self.tape)
        head = ' ' * self.head + '^'
        return "{}\n{}".format(tape, head)


def main():
    pass


if __name__ == '__main__':
    main()
