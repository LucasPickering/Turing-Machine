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

    def set_active_var(self, char):
        """
        Sets the active variable to the given character
        "Take the shot!" from rocketlang
        """
        if ord(char) >= 128:
            raise TypeError("This machine only supports basic ASCII [0, 127]")
        self.active_var = ord(char)

    def incr_var(self):
        """
        Increments the active variable
        "Wow!" from rocketlang
        """
        self.active_var += 1

    def decr_var(self):
        """
        Decrements the active variable
        "Close One!" from rocketlang
        """
        self.active_var -= 1

    def save_active_var(self):
        """
        Copies the active variable to the inactive variable
        "Whoops..." from rocketlang
        """
        self.inactive_var = self.active_var

    def swap(self):
        """
        Swaps the values of the active and inactive variables
        "OMG!" from rocketlang
        """
        temp = self.active_var
        self.active_var = self.inactive_var
        self.inactive_var = temp

    def push_0(self):
        """
        Pushes the value 0 onto the stack
        "Noooo!" from rocketlang
        """
        self.stack.append(0)

    def pop_to_active_var(self):
        """
        Pops the item at the top of the stack, and sets active_var equal to that item
        "Centering..." from rocketlang
        """
        self.active_var = self.stack.pop()

    def push_active_var(self):
        """
        Pushes the value of the active variable onto the stack
        "Defending..." from rocketlang
        """
        self.stack.append(self.active_var)

    def if_statement(self, funcs):
        """
        Runs a single if statement that only executes if the active variable is equal to the
        inactive variable. Each given function is run in order, and must take self as a singular
        argument to enforce that it is a function of this machine.
        "Nice shot!" and "What a save!" from rocketlang - we could require the user to end every
        function list with  "What a save!" equivalent to preserve better correlation with
        rocketlang, but I think this shortcut is okay to take.
        """
        if self.active_var == self.inactive_var:
            for func in funcs:
                func(self)

    def loop(self, funcs):
        """
        Runs a loop that operates the given functions in order while the active variable is >0.
        Each function is assumed to only take self as an argument, to enforce that they are all
        functions of this machine
        "Great pass!" and "Thanks!" from rocketlang - we could require the user to end every
        function list with a "Thanks!" equivalent to preserve better correlation with rocketlang,
        but I think this shortcut is okay to take.
        """
        while self.active_var > 0:
            for func in funcs:
                func(self)


class ExpandedMachine(IntMachine):
    """
    An expanded version of IntMachine that adds convenience functions, but only uses IntMachine's
    functions to do it, so it doesn't add any more functionality.
    """

    def reset_active_var(self):
        """
        Resets the value of the active variable to 0
        """
        self.push_0()  # Push 0 onto the stack
        self.pop_to_active_var()  # Pop 0 from the stack into the active variable


"""
State is the name of the state
"""
Condition = namedtuple('Condition', 'state tape_symbol')

"""
function is one of 'L R W' meaning move left, move right, or write to tape
next_state is the name of the next state to move to
new_symbol is the symbol to write (irrelevant if action isn't 'W')
"""
Action = namedtuple('Action', 'function next_state new_symbol')


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

    VALID_ACTIONS = {'L', 'R', 'W'}

    def __init__(self, accepting_states, initial_state, language=ASCII):
        self.accepting_states = set(accepting_states)
        self.language = language
        self.initial_state = initial_state
        self.instructions = dict()

    def add_instruction(self, condition, action):
        """
        Adds the given condition/action pair to this machine's instruction set.
        """
        # Check the validity of both symbols
        self.__check_symbol(condition.tape_symbol)
        if action.new_symbol:
            self.__check_symbol(action.new_symbol)

        if action.function not in self.VALID_ACTIONS:
            raise ValueError("Invalid function: {}. Must be one of {}".format(action.function,
                                                                              self.VALID_ACTIONS))
        self.instructions[condition] = action  # Add it to the instruction set

    def __check_symbol(self, symbol):
        """
        Checks if the given symbol is in this machine's language. If not, raises a ValueError.
        Also raises a ValueError if the length of the supposed symbol is not 1.
        """
        if len(symbol) != 1:
            raise ValueError("Not a single character: " + symbol)
        if not self.language.contains(symbol):
            raise ValueError("Symbol not in language: " + symbol)

    def __apply_action(self, action):
        if action.function == 'L':
            self.__move_left()
        elif action.function == 'R':
            self.__move_right()
        elif action.function == 'W':
            self.__write_symbol(action.new_symbol)
        self.__

    def __move_left(self):
        self.head -= 1
        # If we hit the left end, extend the tape with an empty square
        if self.head < 0:
            self.tape.insert(0, self.language.empty_symbol)

    def __move_right(self):
        self.head += 1
        # If we hit the right end, extend the tape with an empty square
        if self.head >= len(self.tape):
            self.tape.append(self.language.empty_symbol)

    def __write_symbol(self, symbol):
        self.tape[self.head] = symbol

    def __set_state(self, next_state):
        self.state = next_state

    def __run(self, s):
        """
        Run this machine on the given string, returning True if it is in the language, false if not.
        """
        pass

    def __run_and_print(self, s):
        """
        Run this machine and print ACCEPT or REJECT and the machine's state at halt time.
        """
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
